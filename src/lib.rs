mod cli;
pub mod serve;
pub mod ssg;

use std::{
    fs::{self, File},
    io::Write,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn init_file(path: &str, copy: &[u8]) {
    if fs::exists(path).is_ok_and(|x| !x) {
        let mut file = File::create(path).unwrap();
        file.write_all(copy).unwrap();
    }
}

pub fn init_dir(path: &str) {
    if fs::exists(path).is_ok_and(|x| !x) {
        fs::create_dir(path).unwrap();
    }
}

pub fn is_directory_empty(directory: &str) -> bool {
    let mut entries = fs::read_dir(directory).expect("Could not read directory");
    entries.next().is_none()
}


// We need to validate the current working directory.
// For a directory to be valid we need:
// - Write Permissions
// - toby.toml to exist or
// - an empty directory to populate
pub fn validate_working_directory() {
    validate_working_directory_init(false);
}

// wanted an empty argument for every function that wasn't init lmaoo
pub fn validate_working_directory_init(is_init: bool) {
    let md = fs::metadata(".").unwrap();
    let permissions = md.permissions();
    if permissions.readonly() {
        panic!("No write permissions");
    }

    if fs::exists("toby.toml").is_ok_and(|x| !x) {
        if !is_directory_empty(".") {
            panic!("Directory is not empty");
        }
        if !is_init {
            panic!("Toby SSG has not been initilized");
        }
    }
}

pub fn init() {
    validate_working_directory_init(true);

    let toby = include_bytes!("../toby.toml");
    init_file("toby.toml", toby);

    init_dir("templates");
    init_dir("content");
    init_dir("output");
    init_dir("output/static");

    let base_template = include_bytes!("../templates/base.html");
    let style = include_bytes!("../templates/style.scss");
    let index_md = include_bytes!("../content/index.md");

    init_file("templates/base.html", base_template);
    init_file("templates/style.scss", style);
    init_file("content/index.md", index_md);
}
