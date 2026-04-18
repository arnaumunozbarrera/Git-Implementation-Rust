use std::fs;
use std::path::Path;

use crate::utils::fs_ops;

pub fn init_command() {
    let title = fs::read_to_string("src/cli/title.txt")
        .expect("[ERROR] Could not read ASCII inside file");
    let subtitle = fs::read_to_string("src/cli/subtitle.txt")
        .expect("[ERROR] Could not read ASCII inside file");

    println!("{}\n{}\n", title, subtitle);

    if Path::new(".voor").exists() {
        let paths = [".voor", ".voor/objects", ".voor/refs", ".voor/refs/heads", ".voor/HEAD", ".voor/index", ".voor/.voorignore"];

        for path in paths {
            if !Path::new(path).exists() {
                println!("[ERROR] `.voor` directory already initialized with errors\nDelete it and run `cargo run init` again");
                return;
            }
        }

        println!("[INFO] `.voor` directory already initialized successfully\n");
    } else {
        let result = fs_ops::with_repo_lock("init", || {
            fs::create_dir(".voor").map_err(|error| format!("[ERROR] Unable to create .voor: {}", error))?;
            fs::create_dir(".voor/objects").map_err(|error| format!("[ERROR] Unable to create objects directory: {}", error))?;
            fs::create_dir(".voor/refs").map_err(|error| format!("[ERROR] Unable to create refs directory: {}", error))?;
            fs::create_dir(".voor/refs/heads").map_err(|error| format!("[ERROR] Unable to create heads directory: {}", error))?;
            fs_ops::write_file_atomic(".voor/refs/heads/master", b"")?;
            fs_ops::write_file_atomic(".voor/HEAD", b"ref: refs/heads/master")?;
            fs_ops::write_file_atomic(".voor/index", b"")?;
            fs_ops::write_file_atomic(".voorignore", b".env\n\n.voor/\n/.voor/\n\nCargo.lock\nCargo.toml")?;
            fs_ops::write_file_atomic(".voor/config", b"[remote \"origin\"]\nurl = http://localhost:3000\n")?;

            println!("[INFO] `.voor` directory initialized successfully!\n");
            Ok(())
        });

        if let Err(error) = result {
            println!("{}", error);
        }
    }
}
