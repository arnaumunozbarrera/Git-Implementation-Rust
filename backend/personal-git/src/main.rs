use std::env;
use std::path::Path;

use crate::cli::status::display_status;

mod api;
mod cli;
mod utils;

const MIN_ARGS_LEN_CLI: usize = 4;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("[EXIT] No command typed.\n Try one of this list:");
        println!("\t· init\n\t· cat-file\n\t· hash-object\n");
        return;
    }

    match args[1].as_str() {
        "init" => cli::init::init_command(),
        "cat-file" => {
            if args.len() >= MIN_ARGS_LEN_CLI {
                cli::cat_file::cat_file_command(&args[2], &args[3]);
            } else {
                println!("[ERROR] Not enough arguments provided to execute the `cat-file` command");
            }
        }
        "hash-object" => {
            if args.len() >= MIN_ARGS_LEN_CLI {
                cli::hash_object::hash_object_command(&args[2], &args[3]);
            } else {
                println!("[ERROR] Not enough arguments provided to execute the `hash-object` command");
            }
        }
        "diff" => {
            let old_hash = &args[2];
            let file_path = &args[3];
            cli::diff::diff_by_hash(old_hash, file_path);
        }
        "add" => {
            if args.len() != 3 {
                println!("[EXIT] Unknown argument.\nTry one of these:");
                println!("\t· <file_name>\n\t· . (add all)\n");
                return;
            }

            if args[2] == "." {
                cli::add::add_all(std::path::Path::new("."));
            } else {
                cli::add::add_by_hash(std::path::Path::new(&args[2]));
            }
        }
        "status" => {
            if args.len() != 2 {
                println!("[EXIT] Unknown argument.\nTry without one:");
                return;
            }

            display_status(Path::new("."));
        }
        "commit" => {
            if args.len() != 4 || args[2] != "-m" {
                println!("[EXIT] Unknown argument.\nTry this one:");
                println!("\t· commit -m <commit_message>");
                return;
            }

            cli::commit::commit(&args[3]);
        }
        "branch" => {
            if args.len() == 2 {
                cli::branch::display_branches();
            } else if args.len() == 3 {
                cli::branch::create_branch(&args[2]);
            } else if args.len() == 4 && args[3] == "-D" {
                cli::branch::delete_branch(&args[2]);
            } else {
                println!("[EXIT] Unknown argument.\nTry one of these:");
                println!("\t· branch");
                println!("\t· branch <branch_name>");
                println!("\t· branch <branch_name> -D");
                return;
            }
        }
        "checkout" => {
            if args.len() == 3 {
                cli::checkout::checkout_to_branch(&args[2]);
            } else if args.len() == 4 && args[2] == "-b" {
                cli::checkout::create_branch_and_checkout(&args[3]);
            } else {
                println!("[EXIT] Unknown argument.\nTry one of these:");
                println!("\t· checkout <branch_name>");
                println!("\t· checkout -b <branch_name>");
                return;
            }
        }
        "remote" => {
            if args.len() == 3 {
                cli::remote_server::set_remote(&args[2]);
            } else {
                println!("[EXIT] Unknown argument.\nTry this one:");
                println!("\t· remote <url>");
                return;
            }
        }
        "push" => {
            if args.len() == 2 {
                let branch_name = cli::branch::get_current_branch();
                cli::remote_server::push_branch(&branch_name);
            } else if args.len() == 3 {
                cli::remote_server::push_branch(&args[2]);
            } else {
                println!("[EXIT] Unknown argument.\nTry one of these:");
                println!("\t· push");
                println!("\t· push <branch_name>");
                return;
            }
        }
        "pull" => {
            if args.len() == 2 {
                let branch_name = cli::branch::get_current_branch();
                cli::remote_server::pull_branch(&branch_name);
            } else if args.len() == 3 {
                cli::remote_server::pull_branch(&args[2]);
            } else {
                println!("[EXIT] Unknown argument.\nTry one of these:");
                println!("\t· pull");
                println!("\t· pull <branch_name>");
                return;
            }
        }
        "serve" => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(api::api::api());
        }
        _ => {
            println!("[EXIT] Unknown command.\nTry one of this list:");
            println!("\t· init\n\t· cat-file\n\t· hash-object\n");
        }
    }
}
