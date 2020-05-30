
#[derive(Debug)]
pub struct GraphicsError {
    cause: String,
}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graphics Error caused by '{}'", self.cause)
    }
}

impl std::error::Error for GraphicsError {}

impl From<std::ffi::FromBytesWithNulError> for GraphicsError {
    fn from(e: std::ffi::FromBytesWithNulError) -> Self {
        GraphicsError{ cause: e.to_string() }
    }
}
impl From<std::str::Utf8Error> for GraphicsError {
    fn from(e: std::str::Utf8Error) -> Self {
        GraphicsError{ cause: e.to_string() }
    }
}
impl From<String> for GraphicsError {
    fn from(e: String) -> Self {
        GraphicsError{ cause: e }
    }
}
impl From<&str> for GraphicsError {
    fn from(e: &str) -> Self {
        GraphicsError{ cause: e.to_string() }
    }
}