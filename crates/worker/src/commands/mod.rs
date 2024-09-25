use lazy_static::lazy_static;
use tokio::sync::Semaphore;

pub mod compile;
pub mod errors;
pub mod utils;
pub mod verify;

const PROCESS_SPAWN_LIMIT: usize = 8;
lazy_static! {
    static ref SPAWN_SEMAPHORE: Semaphore = Semaphore::new(PROCESS_SPAWN_LIMIT);
}
