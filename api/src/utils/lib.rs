use std::path::{Path, PathBuf};

pub const SOL_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "sol/temp/");

#[allow(dead_code)]
pub const TEMP_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "temp/");

pub fn get_file_ext(file_path: &str) -> String {
    match file_path.split('.').last() {
        Some(ext) => ext.to_string(),
        None => {
            debug!("LOG: File extension not found");
            "".to_string()
        }
    }
}

pub fn get_file_path(file_path: &String) -> PathBuf {
    match get_file_ext(file_path).to_string() {
        // Leaving this here for potential use with vyper
        ext if ext == "sol" => Path::new(SOL_ROOT).join(file_path),
        _ => Path::new(SOL_ROOT).join(file_path),
    }
}
