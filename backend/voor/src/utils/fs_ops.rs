use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_LOCK_TIMEOUT_MS: u64 = 15_000;
const DEFAULT_LOCK_POLL_MS: u64 = 100;
const STALE_LOCK_TTL_SECS: u64 = 300;

pub struct RepoLockGuard {
    path: PathBuf,
}

impl Drop for RepoLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn with_repo_lock<T, F>(operation: &str, action: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let _guard = acquire_repo_lock(operation, DEFAULT_LOCK_TIMEOUT_MS)?;
    action()
}

pub fn acquire_repo_lock(operation: &str, timeout_ms: u64) -> Result<RepoLockGuard, String> {
    let lock_path = repo_lock_path();
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("[ERROR] Unable to create lock directory: {}", err))?;
    }

    let started = SystemTime::now();
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let content = format!(
                    "pid={}\noperation={}\ncreated_at={}\n",
                    std::process::id(),
                    operation.trim(),
                    timestamp
                );
                file.write_all(content.as_bytes())
                    .map_err(|err| format!("[ERROR] Unable to write lock file: {}", err))?;
                file.sync_all()
                    .map_err(|err| format!("[ERROR] Unable to flush lock file: {}", err))?;

                return Ok(RepoLockGuard { path: lock_path });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                clear_stale_lock_if_needed(&lock_path)?;

                let elapsed = started.elapsed().unwrap_or_default();
                if elapsed >= Duration::from_millis(timeout_ms) {
                    return Err(format!(
                        "[ERROR] Timed out waiting for repository lock while running '{}'",
                        operation.trim()
                    ));
                }

                thread::sleep(Duration::from_millis(DEFAULT_LOCK_POLL_MS));
            }
            Err(error) => {
                return Err(format!("[ERROR] Unable to create repository lock: {}", error));
            }
        }
    }
}

fn repo_lock_path() -> PathBuf {
    if Path::new(".voor").exists() {
        PathBuf::from(".voor/locks/repo.lock")
    } else {
        PathBuf::from(".voor-init.lock")
    }
}

pub fn write_file_atomic(path: impl AsRef<Path>, content: &[u8]) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("[ERROR] Unable to create directory '{}': {}", parent.display(), err))?;
    }

    let temp_path = temp_path_for(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_path)
        .map_err(|err| format!("[ERROR] Unable to create temporary file '{}': {}", temp_path.display(), err))?;

    file.write_all(content)
        .map_err(|err| format!("[ERROR] Unable to write temporary file '{}': {}", temp_path.display(), err))?;
    file.sync_all()
        .map_err(|err| format!("[ERROR] Unable to flush temporary file '{}': {}", temp_path.display(), err))?;
    drop(file);

    if path.exists() {
        fs::remove_file(path)
            .map_err(|err| format!("[ERROR] Unable to replace '{}': {}", path.display(), err))?;
    }

    fs::rename(&temp_path, path).map_err(|err| {
        format!(
            "[ERROR] Unable to move temporary file '{}' into '{}': {}",
            temp_path.display(),
            path.display(),
            err
        )
    })
}

fn temp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "temp".to_string());
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.with_file_name(format!("{}.tmp-{}", file_name, suffix))
}

fn clear_stale_lock_if_needed(lock_path: &Path) -> Result<(), String> {
    if lock_process_is_gone(lock_path)? {
        remove_stale_lock_file(lock_path)?;
        return Ok(());
    }

    let metadata = match fs::metadata(lock_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("[ERROR] Unable to inspect repository lock: {}", error)),
    };
    let modified = metadata
        .modified()
        .map_err(|err| format!("[ERROR] Unable to inspect repository lock timestamp: {}", err))?;
    let age = modified.elapsed().unwrap_or_default();

    if age > Duration::from_secs(STALE_LOCK_TTL_SECS) {
        remove_stale_lock_file(lock_path)?;
    }

    Ok(())
}

fn remove_stale_lock_file(lock_path: &Path) -> Result<(), String> {
    match fs::remove_file(lock_path) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "[ERROR] Unable to remove stale repository lock: {}",
            error
        )),
    }
}

fn lock_process_is_gone(lock_path: &Path) -> Result<bool, String> {
    let content = fs::read_to_string(lock_path)
        .map_err(|err| format!("[ERROR] Unable to read repository lock: {}", err))?;
    let Some(pid) = content
        .lines()
        .find_map(|line| line.strip_prefix("pid="))
        .and_then(|value| value.trim().parse::<u32>().ok())
    else {
        return Ok(false);
    };

    if pid == std::process::id() {
        return Ok(false);
    }

    Ok(!process_exists(pid))
}

#[cfg(target_os = "windows")]
fn process_exists(pid: u32) -> bool {
    let filter = format!("PID eq {}", pid);
    Command::new("tasklist")
        .args(["/FI", &filter, "/NH"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|stdout| stdout.contains(&pid.to_string()))
        .unwrap_or(true)
}

#[cfg(not(target_os = "windows"))]
fn process_exists(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(true)
}
