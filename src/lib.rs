use std::{
    fs::{self, File},
    io::Write,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

// Make our own error that wraps `anyhow::Error`.
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
