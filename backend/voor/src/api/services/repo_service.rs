use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::UserDirs;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sqlx::Row;

use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::{
    Branch, CloneRepoResponse, DeleteActionResponse, InitRepoRequest, InitRepoResponse, Repository,
};
use crate::utils::fs_ops;
use crate::utils::object_store::{self, ObjectType, ParsedObject};
use crate::utils::sync;

const DEFAULT_REMOTE_URL: &str = "http://localhost:3000";

pub async fn get_all_repos(
    client: &SupabaseClient,
    owner_id: &str,
) -> Result<Vec<Repository>, sqlx::Error> {
    let repos = sqlx::query_as::<_, Repository>(
        "SELECT id, name, owner_id, is_private, description, tags, default_branch,
                stars_count, readme_path, theme, created_at::text AS created_at
         FROM repositories
         WHERE owner_id = $1
         ORDER BY created_at ASC",
    )
    .bind(owner_id)
    .fetch_all(&client.pool)
    .await?;

    Ok(repos)
}

pub async fn get_repo_branches(
    client: &SupabaseClient,
    owner_id: &str,
    repo_id: &str,
) -> Result<Vec<Branch>, String> {
    let owner_id = owner_id.trim();
    let repo_id = repo_id.trim();

    if repo_id.is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    let repository_owner: Option<String> =
        sqlx::query_scalar("SELECT owner_id FROM repositories WHERE id = $1")
            .bind(repo_id)
            .fetch_optional(&client.pool)
            .await
            .map_err(|error| {
                format!("[ERROR] Failed to load repository '{}': {}", repo_id, error)
            })?;

    let Some(repository_owner) = repository_owner else {
        return Err(format!("[ERROR] Repository '{}' not found", repo_id));
    };

    if repository_owner != owner_id {
        return Err(format!(
            "[ERROR] User '{}' cannot access repository '{}'",
            owner_id, repo_id
        ));
    }

    sqlx::query_as::<_, Branch>(
        "SELECT id::text AS id, repo_id, name, last_commit_hash, created_at::text AS created_at
         FROM branches
         WHERE repo_id = $1
         ORDER BY
            CASE WHEN name = (SELECT default_branch FROM repositories WHERE id = $1) THEN 0 ELSE 1 END,
            created_at DESC,
            name ASC",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to list branches for '{}': {}", repo_id, error))
}

pub async fn init_repo(
    client: &SupabaseClient,
    owner_id: &str,
    payload: InitRepoRequest,
) -> Result<InitRepoResponse, String> {
    let repo_id = payload.repo_id.trim();
    let name = payload.name.trim();
    let owner_id = owner_id.trim();
    let default_branch = payload.default_branch.trim();
    let description = payload
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let readme_path = payload
        .readme_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if repo_id.is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    if name.is_empty() {
        return Err("[ERROR] Missing name".to_string());
    }

    if owner_id.is_empty() {
        return Err("[ERROR] Missing owner_id".to_string());
    }

    if default_branch.is_empty() {
        return Err("[ERROR] Missing default_branch".to_string());
    }

    let owner_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(owner_id)
        .fetch_one(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to verify owner '{}': {}", owner_id, error))?;

    if !owner_exists {
        return Err(format!("[ERROR] Owner '{}' not found", owner_id));
    }

    let repo_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM repositories WHERE id = $1)")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| {
                format!(
                    "[ERROR] Failed to verify repository '{}': {}",
                    repo_id, error
                )
            })?;

    if repo_exists {
        return Err(format!("[ERROR] Repository '{}' already exists", repo_id));
    }

    sqlx::query(
        "INSERT INTO repositories (id, name, owner_id, is_private, description, tags, default_branch, readme_path, theme) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(repo_id)
    .bind(name)
    .bind(owner_id)
    .bind(payload.is_private)
    .bind(description)
    .bind(payload.tags)
    .bind(default_branch)
    .bind(readme_path)
    .bind(payload.theme)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to initialize repository '{}': {}", repo_id, error))?;

    sqlx::query("INSERT INTO branches (repo_id, name, last_commit_hash) VALUES ($1, $2, $3)")
        .bind(repo_id)
        .bind(default_branch)
        .bind(Option::<String>::None)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to create default branch '{}' for repository '{}': {}",
                default_branch, repo_id, error
            )
        })?;

    Ok(InitRepoResponse {
        message: format!("Initialized remote repository '{}'", repo_id),
        repo_id: repo_id.to_string(),
        database_action: Some(format!(
            "Created repository '{}' with default branch '{}'",
            repo_id, default_branch
        )),
    })
}

pub async fn delete_repo(
    client: &SupabaseClient,
    user_id: &str,
    repo_id: &str,
) -> Result<DeleteActionResponse, String> {
    let user_id = user_id.trim();
    let repo_id = repo_id.trim();

    if repo_id.is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    let owner_id: Option<String> =
        sqlx::query_scalar("SELECT owner_id FROM repositories WHERE id = $1")
            .bind(repo_id)
            .fetch_optional(&client.pool)
            .await
            .map_err(|error| {
                format!("[ERROR] Failed to load repository '{}': {}", repo_id, error)
            })?;

    let Some(owner_id) = owner_id else {
        return Err(format!("[ERROR] Repository '{}' not found", repo_id));
    };

    if owner_id != user_id {
        return Err(format!(
            "[ERROR] User '{}' cannot delete repository '{}'",
            user_id, repo_id
        ));
    }

    sqlx::query("DELETE FROM stars WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete stars for '{}': {}",
                repo_id, error
            )
        })?;

    sqlx::query("DELETE FROM repo_access_logs WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete access logs for '{}': {}",
                repo_id, error
            )
        })?;

    sqlx::query("DELETE FROM commits_metadata WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete commit metadata for '{}': {}",
                repo_id, error
            )
        })?;

    sqlx::query("DELETE FROM branches WHERE repo_id = $1")
        .bind(repo_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete branches for '{}': {}",
                repo_id, error
            )
        })?;

    let result = sqlx::query("DELETE FROM repositories WHERE id = $1 AND owner_id = $2")
        .bind(repo_id)
        .bind(user_id)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to delete repository '{}': {}",
                repo_id, error
            )
        })?;

    if result.rows_affected() == 0 {
        return Err(format!("[ERROR] Repository '{}' not found", repo_id));
    }

    Ok(DeleteActionResponse {
        message: format!("Deleted repository '{}'", repo_id),
        database_action: Some(format!(
            "Removed repository '{}' and related records",
            repo_id
        )),
    })
}

pub async fn clone_repo_to_desktop(
    client: &SupabaseClient,
    user_id: &str,
    repo_id: &str,
    default_branch: Option<&str>,
) -> Result<CloneRepoResponse, String> {
    let user_id = user_id.trim();
    let repo_id = repo_id.trim();

    if repo_id.is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    let repository = sqlx::query_as::<_, Repository>(
        "SELECT id, name, owner_id, is_private, description, tags, default_branch,
                stars_count, readme_path, theme, created_at::text AS created_at
         FROM repositories
         WHERE id = $1",
    )
    .bind(repo_id)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load repository '{}': {}", repo_id, error))?
    .ok_or_else(|| format!("[ERROR] Repository '{}' not found", repo_id))?;

    if repository.owner_id != user_id {
        return Err(format!(
            "[ERROR] User '{}' cannot clone repository '{}'",
            user_id, repo_id
        ));
    }

    let desktop = UserDirs::new()
        .and_then(|dirs| dirs.desktop_dir().map(Path::to_path_buf))
        .ok_or_else(|| "[ERROR] Unable to locate Desktop directory".to_string())?;
    let target = unique_desktop_repo_path(&desktop, &repository.name);
    fs::create_dir_all(&target)
        .map_err(|error| format!("[ERROR] Unable to create '{}': {}", target.display(), error))?;

    let voor_dir = target.join(".voor");
    for directory in [
        voor_dir.as_path(),
        &voor_dir.join("objects"),
        &voor_dir.join("refs"),
        &voor_dir.join("refs").join("heads"),
        &voor_dir.join("locks"),
    ] {
        fs::create_dir_all(directory).map_err(|error| {
            format!(
                "[ERROR] Unable to create '{}': {}",
                directory.display(),
                error
            )
        })?;
    }

    let branch = default_branch
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(repository.default_branch.as_str());
    let branches = sqlx::query("SELECT name, last_commit_hash FROM branches WHERE repo_id = $1")
        .bind(repo_id)
        .fetch_all(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to load branches for '{}': {}",
                repo_id, error
            )
        })?;
    let selected_head = branches
        .iter()
        .find(|row| row.get::<String, _>("name") == branch)
        .and_then(|row| row.get::<Option<String>, _>("last_commit_hash"));
    let hydrated_objects = if let Some(head) = selected_head
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        hydrate_from_local_objects(&target, head).is_ok()
    } else {
        false
    };

    write_file(
        &voor_dir.join("HEAD"),
        format!("ref: refs/heads/{}", branch).as_bytes(),
    )?;
    for row in &branches {
        let branch_name = row.get::<String, _>("name");
        let head = row
            .get::<Option<String>, _>("last_commit_hash")
            .unwrap_or_default();
        let ref_content = if hydrated_objects {
            head
        } else {
            String::new()
        };
        write_file(
            &voor_dir.join("refs").join("heads").join(branch_name),
            ref_content.as_bytes(),
        )?;
    }
    if branches.is_empty() {
        write_file(&voor_dir.join("refs").join("heads").join(branch), b"")?;
    }
    write_file(&voor_dir.join("index"), b"")?;
    write_file(
        &voor_dir.join("config"),
        format!(
            "[remote \"origin\"]\nurl = {}\nrepo_id = {}\nuser_id = {}\n",
            DEFAULT_REMOTE_URL, repo_id, user_id
        )
        .as_bytes(),
    )?;
    write_file(
        &target.join(".voorignore"),
        b".env\n\n.voor/\n/.voor/\n\nCargo.lock\nCargo.toml",
    )?;

    if !hydrated_objects {
        restore_worktree_from_database(client, &target, selected_head.as_deref()).await?;
    }

    Ok(CloneRepoResponse {
        message: format!("Cloned repository '{}' to Desktop", repo_id),
        path: target.display().to_string(),
    })
}

fn hydrate_from_local_objects(target: &Path, head: &str) -> Result<(), String> {
    let objects = sync::collect_encoded_objects(head)?;
    let mut parsed = std::collections::HashMap::new();

    for encoded in objects {
        let full_bytes = sync::decode_object_from_network(&encoded)?;
        write_full_object_to_repo(target, &encoded.hash, &full_bytes)?;
        parsed.insert(
            encoded.hash.clone(),
            object_store::parse_full_object(&encoded.hash, full_bytes)?,
        );
    }

    restore_worktree_from_objects(target, head, &parsed)
}

fn write_full_object_to_repo(target: &Path, hash: &str, full_bytes: &[u8]) -> Result<(), String> {
    let trimmed = hash.trim();
    let (dir, file) = trimmed.split_at(2);
    let path = target.join(".voor").join("objects").join(dir).join(file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("[ERROR] Unable to create '{}': {}", parent.display(), error)
        })?;
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(full_bytes)
        .map_err(|error| format!("[ERROR] Unable to compress object '{}': {}", trimmed, error))?;
    let compressed = encoder
        .finish()
        .map_err(|error| format!("[ERROR] Unable to finalize object '{}': {}", trimmed, error))?;
    fs_ops::write_file_atomic(&path, &compressed)
}

fn restore_worktree_from_objects(
    target: &Path,
    head: &str,
    objects: &std::collections::HashMap<String, ParsedObject>,
) -> Result<(), String> {
    let commit = objects
        .get(head)
        .ok_or_else(|| format!("[ERROR] Missing commit object '{}'", head))?;
    let tree_hash = parse_commit_tree_hash(&commit.content)?;
    restore_tree_from_objects(target, Path::new(""), &tree_hash, objects)
}

fn restore_tree_from_objects(
    target: &Path,
    prefix: &Path,
    tree_hash: &str,
    objects: &std::collections::HashMap<String, ParsedObject>,
) -> Result<(), String> {
    let tree = objects
        .get(tree_hash)
        .ok_or_else(|| format!("[ERROR] Missing tree object '{}'", tree_hash))?;

    for entry in object_store::parse_tree(&tree.content)? {
        let path = prefix.join(&entry.name);
        match entry.object_type {
            ObjectType::Blob => {
                let blob = objects
                    .get(&entry.hash)
                    .ok_or_else(|| format!("[ERROR] Missing blob object '{}'", entry.hash))?;
                write_file(&target.join(path), &blob.content)?;
            }
            ObjectType::Tree => restore_tree_from_objects(target, &path, &entry.hash, objects)?,
            ObjectType::Commit => {}
        }
    }

    Ok(())
}

async fn restore_worktree_from_database(
    client: &SupabaseClient,
    target: &Path,
    head: Option<&str>,
) -> Result<(), String> {
    let Some(head) = head.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    let tree_hash: Option<String> =
        sqlx::query_scalar("SELECT tree_hash FROM commits WHERE hash = $1")
            .bind(head)
            .fetch_optional(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to load head commit '{}': {}", head, error))?;

    let Some(tree_hash) = tree_hash else {
        return Ok(());
    };

    restore_db_tree(client, target, Path::new(""), &tree_hash).await
}

async fn restore_db_tree(
    client: &SupabaseClient,
    target: &Path,
    prefix: &Path,
    tree_hash: &str,
) -> Result<(), String> {
    let mut stack = vec![(prefix.to_path_buf(), tree_hash.to_string())];

    while let Some((current_prefix, current_tree)) = stack.pop() {
        let rows = sqlx::query(
            "SELECT name, type, hash, mode FROM tree_entries WHERE tree_hash = $1 ORDER BY name ASC",
        )
        .bind(&current_tree)
        .fetch_all(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to load tree '{}': {}", current_tree, error))?;

        for row in rows {
            let name = row.get::<String, _>("name");
            let entry_type = row.get::<String, _>("type");
            let hash = row.get::<String, _>("hash");
            let path = current_prefix.join(name);

            if entry_type == "tree" {
                stack.push((path, hash));
            } else {
                let content: Option<Vec<u8>> =
                    sqlx::query_scalar("SELECT content FROM blobs WHERE hash = $1")
                        .bind(&hash)
                        .fetch_optional(&client.pool)
                        .await
                        .map_err(|error| {
                            format!("[ERROR] Failed to load blob '{}': {}", hash, error)
                        })?;
                if let Some(content) = content {
                    write_file(&target.join(path), &content)?;
                }
            }
        }
    }

    Ok(())
}

fn parse_commit_tree_hash(content: &[u8]) -> Result<String, String> {
    let commit_text = String::from_utf8(content.to_vec())
        .map_err(|error| format!("[ERROR] Invalid commit content: {}", error))?;
    commit_text
        .lines()
        .find_map(|line| line.strip_prefix("tree "))
        .map(str::trim)
        .map(str::to_string)
        .ok_or_else(|| "[ERROR] Commit missing tree hash".to_string())
}

fn unique_desktop_repo_path(desktop: &Path, name: &str) -> PathBuf {
    let base_name = sanitize_folder_name(name);
    let first = desktop.join(&base_name);
    if !first.exists() {
        return first;
    }

    for index in 2..1000 {
        let candidate = desktop.join(format!("{}-{}", base_name, index));
        if !candidate.exists() {
            return candidate;
        }
    }

    desktop.join(format!("{}-{}", base_name, uuid::Uuid::new_v4()))
}

fn sanitize_folder_name(name: &str) -> String {
    let sanitized = name
        .trim()
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            _ => character,
        })
        .collect::<String>()
        .trim_matches([' ', '.'])
        .to_string();

    if sanitized.is_empty() {
        "voor-repository".to_string()
    } else {
        sanitized
    }
}

fn write_file(path: &Path, content: &[u8]) -> Result<(), String> {
    fs_ops::write_file_atomic(path.to_string_lossy().as_ref(), content)
}
