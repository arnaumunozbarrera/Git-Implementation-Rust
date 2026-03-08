use std::fs;
use std::path::Path;

pub fn init_command() {
    if Path::new(".voor").exists() {
        println!("[INFO] `.voor` directory already initialized\n");
    } else {
        fs::create_dir(".voor").unwrap();
        fs::create_dir(".voor/objects").unwrap();
        fs::create_dir(".voor/ref").unwrap();
        fs::write(".voor/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("[INFO] `.voor` directory initialized successfully!\n");
    }
}