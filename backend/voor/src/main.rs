use std::env;
use std::path::Path;

use clap::{ArgAction, Args, Parser, Subcommand};

use crate::cli::status::display_status;
use crate::utils::repo;

mod api;
mod cli;
mod utils;

#[derive(Debug, Parser)]
#[command(
    name = "voor",
    version,
    about = "A Git-like distributed version control CLI implemented in Rust",
    arg_required_else_help = true
)]
struct Cli {
    #[arg(short = 'C', long = "repo-dir", global = true, value_name = "PATH")]
    repo_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    CatFile(CatFileArgs),
    HashObject(HashObjectArgs),
    Diff(DiffArgs),
    Add(AddArgs),
    Status,
    Commit(CommitArgs),
    Branch(BranchArgs),
    Checkout(CheckoutArgs),
    Remote(RemoteArgs),
    Login(LoginArgs),
    Logout,
    InitRemote(InitRemoteArgs),
    Push(BranchTargetArgs),
    Pull(BranchTargetArgs),
    SyncDb(BranchTargetArgs),
    Serve,
}

#[derive(Debug, Args)]
struct CatFileArgs {
    #[arg(short = 'p', long = "pretty-print", action = ArgAction::SetTrue)]
    pretty_print: bool,
    hash: String,
}

#[derive(Debug, Args)]
struct HashObjectArgs {
    #[arg(short = 'w', action = ArgAction::SetTrue)]
    write: bool,
    #[arg(long = "sha1", action = ArgAction::SetTrue)]
    sha1: bool,
    #[arg(long = "sha256", action = ArgAction::SetTrue)]
    sha256: bool,
    file_path: String,
}

#[derive(Debug, Args)]
struct DiffArgs {
    old_hash: String,
    file_path: String,
}

#[derive(Debug, Args)]
struct AddArgs {
    path: String,
}

#[derive(Debug, Args)]
struct CommitArgs {
    #[arg(short = 'm', long = "message")]
    message: String,
}

#[derive(Debug, Args)]
struct BranchArgs {
    branch_name: Option<String>,
    #[arg(short = 'D', action = ArgAction::SetTrue)]
    delete: bool,
}

#[derive(Debug, Args)]
struct CheckoutArgs {
    branch_name: Option<String>,
    #[arg(short = 'b', long = "create")]
    create: Option<String>,
}

#[derive(Debug, Args)]
struct RemoteArgs {
    url: String,
}

#[derive(Debug, Args)]
struct LoginArgs {
    clerk_jwt: String,
}

#[derive(Debug, Args)]
struct InitRemoteArgs {
    branch_name: Option<String>,
}

#[derive(Debug, Args)]
struct BranchTargetArgs {
    branch_name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(repo_dir) = cli.repo_dir.as_deref() {
        if let Err(error) = env::set_current_dir(repo_dir) {
            println!("[ERROR] Unable to switch to '{}': {}", repo_dir, error);
            return;
        }
    }

    let invocation_dir = env::current_dir().ok();

    let repo_root = if requires_repository(&cli.command) {
        match repo::switch_to_repo_root() {
            Ok(path) => Some(path),
            Err(error) => {
                println!("{}", error);
                return;
            }
        }
    } else {
        None
    };

    match cli.command {
        Commands::Init => cli::init::init_command(),
        Commands::CatFile(args) => {
            if !args.pretty_print {
                println!("[ERROR] Unknown argument. Did you mean `-p`?");
                return;
            }
            cli::cat_file::cat_file_command("-p", &args.hash);
        }
        Commands::HashObject(args) => {
            let mode = if args.sha256 {
                "--sha256"
            } else if args.sha1 || args.write {
                "-w"
            } else {
                "-w"
            };
            let file_path = resolve_user_path(&args.file_path, invocation_dir.as_deref());
            cli::hash_object::hash_object_command(mode, &file_path);
        }
        Commands::Diff(args) => {
            let file_path = resolve_user_path(&args.file_path, invocation_dir.as_deref());
            cli::diff::diff_by_hash(&args.old_hash, &file_path);
        }
        Commands::Add(args) => {
            let Some(repo_root) = repo_root.as_deref() else {
                println!("[ERROR] Unable to determine repository root");
                return;
            };

            let staged_path = match repo_relative_path(&args.path, invocation_dir.as_deref(), repo_root) {
                Ok(path) => path,
                Err(error) => {
                    println!("{}", error);
                    return;
                }
            };

            if staged_path == "." {
                cli::add::add_all(Path::new(&staged_path));
            } else {
                cli::add::add_by_hash(Path::new(&staged_path));
            }
        }
        Commands::Status => display_status(Path::new(".")),
        Commands::Commit(args) => cli::commit::commit(&args.message),
        Commands::Branch(args) => match (args.branch_name.as_deref(), args.delete) {
            (None, false) => cli::branch::display_branches(),
            (Some(branch_name), false) => cli::branch::create_branch(branch_name),
            (Some(branch_name), true) => cli::branch::delete_branch(branch_name),
            (None, true) => println!("[ERROR] Missing branch name for deletion"),
        },
        Commands::Checkout(args) => {
            if let Some(branch_name) = args.create.as_deref() {
                cli::checkout::create_branch_and_checkout(branch_name);
            } else if let Some(branch_name) = args.branch_name.as_deref() {
                cli::checkout::checkout_to_branch(branch_name);
            } else {
                println!("[ERROR] Missing branch name. Use `voor checkout <branch>` or `voor checkout -b <branch>`");
            }
        }
        Commands::Remote(args) => cli::remote_server::set_remote(&args.url),
        Commands::Login(args) => cli::remote_server::login(&args.clerk_jwt),
        Commands::Logout => cli::remote_server::logout(),
        Commands::InitRemote(args) => cli::remote_server::init_remote(args.branch_name.as_deref()),
        Commands::Push(args) => {
            let branch_name = args
                .branch_name
                .unwrap_or_else(cli::branch::get_current_branch);
            cli::remote_server::push_branch(&branch_name);
        }
        Commands::Pull(args) => {
            let branch_name = args
                .branch_name
                .unwrap_or_else(cli::branch::get_current_branch);
            cli::remote_server::pull_branch(&branch_name);
        }
        Commands::SyncDb(args) => cli::remote_server::sync_db(args.branch_name.as_deref()),
        Commands::Serve => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(api::api::api());
        }
    }
}

fn requires_repository(command: &Commands) -> bool {
    !matches!(command, Commands::Init | Commands::Serve | Commands::Login(_) | Commands::Logout)
}

fn resolve_user_path(path: &str, invocation_dir: Option<&Path>) -> String {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return candidate.to_string_lossy().to_string();
    }

    invocation_dir
        .unwrap_or_else(|| Path::new("."))
        .join(candidate)
        .to_string_lossy()
        .to_string()
}

fn repo_relative_path(path: &str, invocation_dir: Option<&Path>, repo_root: &Path) -> Result<String, String> {
    if path == "." {
        let base = invocation_dir.unwrap_or(repo_root);
        let relative = base.strip_prefix(repo_root).unwrap_or(Path::new(""));
        if relative.as_os_str().is_empty() {
            return Ok(".".to_string());
        }
        return Ok(relative.to_string_lossy().replace('\\', "/"));
    }

    let absolute = Path::new(&resolve_user_path(path, invocation_dir)).to_path_buf();
    let relative = absolute.strip_prefix(repo_root).map_err(|_| {
        format!(
            "[ERROR] Path '{}' is outside the current repository '{}'",
            path,
            repo_root.display()
        )
    })?;

    Ok(relative.to_string_lossy().replace('\\', "/"))
}
