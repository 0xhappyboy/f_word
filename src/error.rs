//! Error types for the Word document engine.
#[derive(Debug)]
pub enum FWordError {
    /// I/O failure when reading or writing files.
    Io(std::io::Error),
    /// The archive could not be opened or a required entry is missing.
    Zip(zip::result::ZipError),
    /// The XML content is malformed or does not match the expected schema.
    Xml(String),
    /// The requested operation is not valid for the current document state.
    Invalid(String),
}
impl std::fmt::Display for FWordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FWordError::Io(e) => write!(f, "io error: {}", e),
            FWordError::Zip(e) => write!(f, "zip error: {}", e),
            FWordError::Xml(e) => write!(f, "xml error: {}", e),
            FWordError::Invalid(e) => write!(f, "invalid operation: {}", e),
        }
    }
}
impl std::error::Error for FWordError {}
impl From<std::io::Error> for FWordError {
    fn from(e: std::io::Error) -> Self {
        FWordError::Io(e)
    }
}
impl From<zip::result::ZipError> for FWordError {
    fn from(e: zip::result::ZipError) -> Self {
        FWordError::Zip(e)
    }
}
pub type FWordResult<T> = Result<T, FWordError>;
