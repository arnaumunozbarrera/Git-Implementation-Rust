// Import libraries
use std::fs;

pub fn read_head() -> String {
    fs::read_to_string(".voor/HEAD").expect("[ERROR] Unable to read HEAD");
}

pub fn get_head_ref() -> String {
    let head = read_head();

    head.strip_prefix("ref: ")
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn read_head_target() -> String {
    let head_ref = get_head_ref();
    let path = format!(".voor/{}", head_ref);

    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn update_ref(reference: &str, hash_content: &str) {
    let path = format!(".voor/{}", reference);
    
    fs::write(path, format!("{}", hash)).expect("[ERROR] Unable to update ref");
}

pub fn update_head_branch(branch: &str) {
    fs::write(".voor/HEAD", format!("ref: refs/heads/{}", branch)).expect("[ERROR] Unable to update HEAD");
}