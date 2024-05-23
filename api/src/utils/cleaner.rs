use rocket::tokio;
use std::path::Path;

pub struct AutoCleanUp<'a> {
    pub(crate) dirs: Vec<&'a str>,
}

impl Drop for AutoCleanUp<'_> {
    fn drop(&mut self) {
        self.clean_up_sync();
    }
}

impl AutoCleanUp<'_> {
    pub async fn clean_up(&self) {
        for path in self.dirs.iter() {
            println!("Removing path: {:?}", path);

            // check if the path exists
            if !Path::new(path).exists() {
                continue;
            }

            if let Err(e) = tokio::fs::remove_dir_all(path).await {
                tracing::info!("Failed to remove file: {:?}", e);
            }
        }
    }

    pub fn clean_up_sync(&self) {
        for path in self.dirs.iter() {
            println!("Removing path: {:?}", path);

            // check if the path exists
            if !Path::new(path).exists() {
                continue;
            }

            if let Err(e) = std::fs::remove_dir_all(path) {
                tracing::info!("Failed to remove file: {:?}", e);
            }
        }
    }
}
