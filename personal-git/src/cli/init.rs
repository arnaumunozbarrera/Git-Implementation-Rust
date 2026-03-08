use std::fs;
use std::path::Path;

pub fn init_command() {
    let title = fs::read_to_string("src/cli/title.txt")
        .expect("[ERROR] Could not read ASCII inside file");
    let subtitle = fs::read_to_string("src/cli/subtitle.txt")
        .expect("[ERROR] Could not read ASCII inside file");

    println!("{}\n{}\n", title, subtitle);

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