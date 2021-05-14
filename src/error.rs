use thiserror::Error;

use crate::graphics::ShaderError;
#[derive(Error, Debug)]
pub enum AyudeError {
    #[error(transparent)]
    Shader(#[from] ShaderError)
}

// impl std::fmt::Display for AyudeError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Graphics Error caused by '{}'", self.cause)
//     }
// }

// impl std::error::Error for AyudeError {}

// impl From<std::io::Error> for AyudeError {
//     fn from(e: std::io::Error) -> Self {
//         AyudeError {
//             cause: e.to_string(),
//         }
//     }
// }
// impl From<std::str::Utf8Error> for AyudeError {
//     fn from(e: std::str::Utf8Error) -> Self {
//         AyudeError {
//             cause: e.to_string(),
//         }
//     }
// }
// impl From<image::error::ImageError> for AyudeError {
//     fn from(e: image::error::ImageError) -> Self {
//         AyudeError {
//             cause: e.to_string(),
//         }
//     }
// }
// impl From<std::ffi::FromBytesWithNulError> for AyudeError {
//     fn from(e: std::ffi::FromBytesWithNulError) -> Self {
//         AyudeError {
//             cause: e.to_string(),
//         }
//     }
// }
// impl From<String> for AyudeError {
//     fn from(e: String) -> Self {
//         AyudeError { cause: e }
//     }
// }
// impl From<&str> for AyudeError {
//     fn from(e: &str) -> Self {
//         AyudeError {
//             cause: e.to_string(),
//         }
//     }
// }
