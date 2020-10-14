pub use super::errorkind::ErrorKind;
use serde_json::Error as JsonError;
#[warn(clippy::all)]
use std::{
    fmt::{Display, Formatter},
    io::Error as IoError,
};

use reqwest::Error as ReqwestError;

#[derive(Debug)]
pub struct Error {
    context: String,
    errorkind: ErrorKind,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new<T>(context: T, errorkind: ErrorKind) -> Self
    where
        T: AsRef<str>,
    {
        Error {
            context: context.as_ref().to_owned(),
            errorkind,
        }
    }

    pub fn context(&self) -> &str {
        &self.context
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.errorkind {
            ErrorKind::BadArgs => write!(f, "Called with bad arguments: {}", self.context),
            ErrorKind::EmptyData => write!(f, "Unexpectedly empty data: {}", self.context),
            ErrorKind::NoToken => write!(
                f,
                "The environmental variable GITHUB_API_TOKEN must contain your GitHub API \
                token. Context: {}",
                self.context
            ),
            ErrorKind::Io(io) => write!(f, "IO Error: {}\nContext: {}", io, self.context),
            ErrorKind::Json(json) => write!(
                f,
                "Serde JSON error: {:?}; Context: {context}",
                json,
                context = self.context()
            ),
            ErrorKind::Reqwest(reqw) => {
                write!(f, "Reqwest error: {}\nContext {}", reqw, self.context)
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<JsonError> for Error {
    // The classify() function would provide us with a reasonable default for context if ? passes
    // the error up.
    fn from(json: JsonError) -> Self {
        Error::new(format!("{:?}", json.classify()), ErrorKind::Json(json))
    }
}

impl From<IoError> for Error {
    fn from(io: IoError) -> Self {
        Error::new("Empty context", ErrorKind::Io(io))
    }
}

impl From<ReqwestError> for Error {
    fn from(reqw: ReqwestError) -> Self {
        Error::new("Empty context", ErrorKind::Reqwest(reqw))
    }
}
