use lazy_static::lazy_static;
use std::path::PathBuf;
use tokio::sync::Semaphore;

pub mod compile;
pub mod errors;
pub mod verify;

const PROCESS_SPAWN_LIMIT: usize = 8;
lazy_static! {
    static ref SPAWN_SEMAPHORE: Semaphore = Semaphore::new(PROCESS_SPAWN_LIMIT);
}

#[derive()]
pub struct CompilationFile {
    pub file_path: PathBuf,
    pub file_content: Vec<u8>,
}
