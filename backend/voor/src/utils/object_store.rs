use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest as Sha1Digest, Sha1};

use crate::utils::fs_ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
        }
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            _ => Err(format!("[ERROR] Unsupported object type '{}'", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedObject {
    pub object_type: ObjectType,
    pub content: Vec<u8>,
    pub full_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
    pub object_type: ObjectType,
}

pub fn object_path(hash: &str) -> String {
    let trimmed = hash.trim();
    let (dir, file) = trimmed.split_at(2);
    format!(".voor/objects/{}/{}", dir, file)
}

pub fn serialize_object(object_type: ObjectType, content: &[u8]) -> Vec<u8> {
    let mut full = format!("{} {}\0", object_type.as_str(), content.len()).into_bytes();
    full.extend_from_slice(content);
    full
}

pub fn compute_hash(full: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(full);
    format!("{:x}", hasher.finalize())
}

pub fn hash_object(object_type: ObjectType, content: &[u8]) -> (String, Vec<u8>) {
    let full = serialize_object(object_type, content);
    let hash = compute_hash(&full);
    (hash, full)
}

pub fn write_full_object(hash: &str, full_bytes: &[u8]) -> Result<(), String> {
    let path = object_path(hash);

    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("[ERROR] Unable to create object directory: {}", err))?;
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(full_bytes)
        .map_err(|err| format!("[ERROR] Unable to compress object: {}", err))?;
    let compressed = encoder
        .finish()
        .map_err(|err| format!("[ERROR] Unable to finalize compression: {}", err))?;

    fs_ops::write_file_atomic(&path, &compressed)
        .map_err(|err| format!("[ERROR] Unable to write object file: {}", err))
}

pub fn write_object(object_type: ObjectType, content: &[u8]) -> Result<String, String> {
    let (hash, full_bytes) = hash_object(object_type, content);
    write_full_object(&hash, &full_bytes)?;
    Ok(hash)
}

pub fn read_full_object(hash: &str) -> Result<Vec<u8>, String> {
    let compressed = fs::read(object_path(hash))
        .map_err(|_| format!("[ERROR] Missing object '{}'", hash.trim()))?;

    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|err| format!("[ERROR] Unable to decompress object '{}': {}", hash.trim(), err))?;

    Ok(decompressed)
}

pub fn parse_full_object(hash: &str, full_bytes: Vec<u8>) -> Result<ParsedObject, String> {
    let null_pos = full_bytes
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| format!("[ERROR] Invalid object header for '{}'", hash.trim()))?;

    let header = std::str::from_utf8(&full_bytes[..null_pos])
        .map_err(|err| format!("[ERROR] Invalid object header for '{}': {}", hash.trim(), err))?;

    let (raw_type, _) = header
        .split_once(' ')
        .ok_or_else(|| format!("[ERROR] Invalid object header for '{}'", hash.trim()))?;

    Ok(ParsedObject {
        object_type: ObjectType::from_str(raw_type)?,
        content: full_bytes[null_pos + 1..].to_vec(),
        full_bytes,
    })
}

pub fn read_object(hash: &str) -> Result<ParsedObject, String> {
    let full_bytes = read_full_object(hash)?;
    parse_full_object(hash, full_bytes)
}

pub fn raw_to_hex(raw: &[u8]) -> String {
    raw.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn hex_to_raw(hash: &str) -> Result<Vec<u8>, String> {
    let trimmed = hash.trim();
    if trimmed.len() != 40 {
        return Err(format!("[ERROR] Invalid hash '{}'", trimmed));
    }

    let mut raw = Vec::with_capacity(20);
    let mut index = 0;
    while index < trimmed.len() {
        let byte = u8::from_str_radix(&trimmed[index..index + 2], 16)
            .map_err(|err| format!("[ERROR] Invalid hash '{}': {}", trimmed, err))?;
        raw.push(byte);
        index += 2;
    }

    Ok(raw)
}

pub fn serialize_tree(entries: &[TreeEntry]) -> Result<Vec<u8>, String> {
    let mut content = Vec::new();

    for entry in entries {
        content.extend_from_slice(entry.mode.as_bytes());
        content.push(b' ');
        content.extend_from_slice(entry.name.as_bytes());
        content.push(0);
        content.extend_from_slice(&hex_to_raw(&entry.hash)?);
    }

    Ok(content)
}

pub fn parse_tree(content: &[u8]) -> Result<Vec<TreeEntry>, String> {
    let mut entries = Vec::new();
    let mut cursor = 0usize;

    while cursor < content.len() {
        let mode_end = content[cursor..]
            .iter()
            .position(|byte| *byte == b' ')
            .ok_or_else(|| "[ERROR] Invalid tree format: missing mode".to_string())?
            + cursor;

        let mode = std::str::from_utf8(&content[cursor..mode_end])
            .map_err(|err| format!("[ERROR] Invalid tree mode: {}", err))?
            .to_string();

        let name_start = mode_end + 1;
        let name_end = content[name_start..]
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| "[ERROR] Invalid tree format: missing name terminator".to_string())?
            + name_start;

        let name = std::str::from_utf8(&content[name_start..name_end])
            .map_err(|err| format!("[ERROR] Invalid tree entry name: {}", err))?
            .to_string();

        let hash_start = name_end + 1;
        let hash_end = hash_start + 20;
        if hash_end > content.len() {
            return Err("[ERROR] Invalid tree format: truncated hash".to_string());
        }

        entries.push(TreeEntry {
            mode: mode.clone(),
            name,
            hash: raw_to_hex(&content[hash_start..hash_end]),
            object_type: if mode == "40000" {
                ObjectType::Tree
            } else {
                ObjectType::Blob
            },
        });

        cursor = hash_end;
    }

    Ok(entries)
}
