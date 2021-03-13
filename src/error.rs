#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    OPSignInError(String),
    ItemQueryError(String),
    ItemDeserializeError(serde_json::error::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Self::ItemDeserializeError(e)
    }
}
