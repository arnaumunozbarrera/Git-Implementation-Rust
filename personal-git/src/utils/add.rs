use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;

pub fn write_index(hash: &str, path: &Path) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(".voor/index")
        .unwrap();

    writeln!(file, "{} {}", hash, path.display()).unwrap();
}