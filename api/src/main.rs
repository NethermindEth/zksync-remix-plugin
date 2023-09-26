#[macro_use]
extern crate rocket;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rocket::data::{Data, ToByteUnit};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Method, Status};
use rocket::tokio::fs;
use rocket::{Request, Response};

mod utils;
use utils::lib::{get_file_ext, get_file_path};

#[derive(Default)]

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if request.method() == Method::Options {
            response.set_status(Status::NoContent);
            response.set_header(Header::new(
                "Access-Control-Allow-Methods",
                "POST, PATCH, GET, DELETE",
            ));
            response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        }

        // Take the Plugin App URL from the env variable, if set
        match env::var("REACT_APP_URL") {
            Ok(v) => {
                response.set_header(Header::new(
                    "Access-Control-Allow-Origin",
                    v
                ));
            },
            Err(e) => {
                response.set_header(Header::new(
                    "Access-Control-Allow-Origin",
                    "https://cairo-remix-test.nethermind.io"
                ));
        
            }
        }

        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}


#[post("/save_code/<remix_file_path..>", data = "<file>")]
async fn save_code(file: Data<'_>, remix_file_path: PathBuf) -> String {
    let remix_file_path = match remix_file_path.to_str() {
        Some(path) => path.to_string(),
        None => {
            return "".to_string();
        }
    };

    let file_path = get_file_path(&remix_file_path);

    // create file directory from file path
    match file_path.parent() {
        Some(parent) => match fs::create_dir_all(parent).await {
            Ok(_) => {
                println!("LOG: Created directory: {:?}", parent);
            }
            Err(e) => {
                println!("LOG: Error creating directory: {:?}", e);
            }
        },
        None => {
            println!("LOG: Error creating directory");
        }
    }

    // Modify to zip and unpack.
    let saved_file = file.open(128_i32.gibibytes()).into_file(&file_path).await;

    match saved_file {
        Ok(_) => {
            println!("LOG: File saved successfully");
            match file_path.to_str() {
                Some(path) => path.to_string(),
                None => "".to_string(),
            }
        }
        Err(e) => {
            println!("LOG: Error saving file: {:?}", e);
            "".to_string()
            // set the response with not ok code.
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CompileResponse {
    pub status: String,
    pub message: String,
    pub file_content: String,
}

#[get("/compile/<remix_file_path..>")]
async fn compile(remix_file_path: PathBuf) -> Json<CompileResponse> {
    let remix_file_path_str = match remix_file_path.to_str() {
        Some(path) => path.to_string(),
        None => {
            return Json(CompileResponse {
                file_content: "".to_string(),
                message: "File path not found".to_string(),
                status: "FileNotFound".to_string(),
            });
        }
    };

    let file_path = format!("../upload/{}", remix_file_path_str);

    let output_path = format!("./compiled/{}.json", remix_file_path_str.trim_end_matches(".sol"));

    let mut compile = Command::new("./zksolc");
    compile
        .arg("--solc")
        .arg("./solc-linux-amd64-v0.8.19+commit.7dd6d404")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-O")
        .arg("3")
        .arg("--overwrite")
        .arg("--combined-json")
        .arg("abi")
        .stderr(Stdio::piped());

    println!("LOG: Running command: {:?}", compile);

    let output = compile.spawn()
        .expect("Failed to execute zksolc")
        .wait_with_output()
        .expect("Failed to wait on child");

    Json(CompileResponse {
        file_content: match fs::read_to_string(&output_path).await {
            Ok(json) => json,
            Err(e) => e.to_string(),
        },
        message: String::from_utf8(output.stderr)
            .unwrap()
            .replace(&file_path, &remix_file_path_str)
            .replace(&output_path, &format!("compiled/{}.json", remix_file_path_str.trim_end_matches(".sol"))),
        status: match output.status.code() {
            Some(0) => "Success".to_string(),
            Some(_) => "CompilationFailed".to_string(),
            None => "UnknownError".to_string(),
        },
    })
}

#[derive(Serialize, Deserialize)]
pub struct FileContentMap {
    pub file_name: String,
    pub file_content: String,
}

#[derive(Serialize, Deserialize)]
pub struct ScarbCompileResponse {
    pub status: String,
    pub message: String,
    pub file_content_map_array: Vec<FileContentMap>,
}

fn get_files_recursive(base_path: &Path) -> Vec<FileContentMap> {
    let mut file_content_map_array: Vec<FileContentMap> = Vec::new();

    if base_path.is_dir() {
        for entry in base_path.read_dir().unwrap() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    file_content_map_array.extend(get_files_recursive(&path));
                } else {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                        let file_content = content;
                        let file_content_map = FileContentMap {
                            file_name,
                            file_content,
                        };
                        file_content_map_array.push(file_content_map);
                    }
                }
            }
        }
    }

    file_content_map_array
}

#[get("/compile-scarb/<remix_file_path..>")]
async fn scarb_compile(remix_file_path: PathBuf) -> Json<ScarbCompileResponse> {
    let remix_file_path = match remix_file_path.to_str() {
        Some(path) => path.to_string(),
        None => {
            return Json(ScarbCompileResponse {
                file_content_map_array: vec![],
                message: "File path not found".to_string(),
                status: "FileNotFound".to_string(),
            });
        }
    };

    let file_path = get_file_path(&remix_file_path);

    let mut compile = Command::new("scarb");
    compile.current_dir(&file_path);

    let result = compile
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute scarb build");

    println!("LOG: ran command:{:?}", compile);

    let output = result.wait_with_output().expect("Failed to wait on child");

    Json(ScarbCompileResponse {
        file_content_map_array: get_files_recursive(&file_path.join("target/dev")),
        message: String::from_utf8(output.stdout)
            .unwrap()
            .replace(&file_path.to_str().unwrap().to_string(), &remix_file_path)
            + &String::from_utf8(output.stderr)
                .unwrap()
                .replace(&file_path.to_str().unwrap().to_string(), &remix_file_path),
        status: match output.status.code() {
            Some(0) => "Success".to_string(),
            Some(_) => "SierraCompilationFailed".to_string(),
            None => "UnknownError".to_string(),
        },
    })
}

// Read the version from the cairo Cargo.toml file.
#[get("/compiler_version")]
async fn compiler_version() -> String {
    let mut version_caller = Command::new("./zksolc");
    match String::from_utf8(
        version_caller
            .arg("--version")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute cairo-compile")
            .wait_with_output()
            .expect("Failed to wait on child")
            .stdout,
    ) {
        Ok(version) => version,
        Err(e) => e.to_string(),
    }
}

#[get("/health")]
async fn health() -> &'static str {
    "OK"
}

#[get("/")]
async fn who_is_this() -> &'static str {
    "Who are you?"
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(CORS).mount(
        "/",
        routes![
            save_code,
            compile,
            compiler_version,
            health,
            who_is_this,
        ],
    )
}
