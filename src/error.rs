use super::errorkind::ErrorKind;
#[warn(clippy::all)]
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
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
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.errorkind {
            ErrorKind::BadArgs => write!(f, "Called with bad arguments: {}", self.context),
            //_ => write!(f, "Unreachable error? Context: {}", self.context),
        }
    }
}

impl std::error::Error for Error {}
