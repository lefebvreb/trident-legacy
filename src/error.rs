use ocl::error::Error as OclError;

pub struct Error(Option<OclError>);

impl From<OclError> for Error {
    fn from(e: OclError) -> Error {
        Error(Some(e))
    }
}

impl Error {
    pub fn new() -> Error {
        Error(None)
    }
}

pub type QRustResult<T> = Result<T, Error>;