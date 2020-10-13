#[warn(clippy::all)]
use serde_json::Error as JsonError;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum ErrorKind {
    BadArgs,
    EmptyData,
    Json(JsonError),
    Io(IoError),
}
