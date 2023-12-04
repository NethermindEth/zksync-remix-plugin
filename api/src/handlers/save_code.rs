use crate::types::{ApiError, Result};
use crate::utils::lib::{
    get_file_path, init_parent_directories, path_buf_to_string, ALLOWED_VERSIONS,
};
use rocket::data::ToByteUnit;
use rocket::Data;
use std::path::PathBuf;
use tracing::info;

#[post("/save_code/<version>/<remix_file_path..>", data = "<file>")]
pub async fn save_code(file: Data<'_>, version: String, remix_file_path: PathBuf) -> String {
    info!("/save_code/{:?}", remix_file_path);
    do_save_code(file, version, remix_file_path)
        .await
        .unwrap_or_else(|e| e.to_string())
}

/// Upload a data file
///
pub async fn do_save_code(
    file: Data<'_>,
    version: String,
    remix_file_path: PathBuf,
) -> Result<String> {
    if !ALLOWED_VERSIONS.contains(&version.as_str()) {
        return Err(ApiError::VersionNotSupported(version));
    }

    let remix_file_path = path_buf_to_string(remix_file_path.clone())?;

    let file_path = get_file_path(&version, &remix_file_path);

    let file_name = file_path.file_name().unwrap().to_str().unwrap();

    // Trim .sol extension
    let file_name_without_ext = file_name.trim_end_matches(".sol");

    // make /<file_name_without_ext>/<file_name>.sol
    let file_path = file_path
        .parent()
        .unwrap()
        .join(file_name_without_ext)
        .join(file_name);

    // create file directory from file path
    init_parent_directories(file_path.clone()).await;

    // Modify to zip and unpack.
    let _ = file
        .open(128_i32.gibibytes())
        .into_file(&file_path)
        .await
        .map_err(ApiError::FailedToSaveFile)?;

    file_path
        .to_str()
        .ok_or(ApiError::FailedToParseString)
        .map(|s| s.to_string())
}
