/////////////////////////////////////////////////////////////////
///  NOTE:
///  - Custom error handling (UiError and UiResult) is defined
///    but not fully integrated throughout the code yet.
///  - Future improvements will focus on robust error handling,
///    graceful recovery, and better logging.
/// 
/// Please treat this as a work-in-progress
/////////////////////////////////////////////////////////////////

use std::fmt;

use slint::PlatformError;


#[derive(Debug)]
pub(crate) enum UiError {
    _WindowCreation(String),
    _WindowOperation(String),
    Platform(PlatformError),
    _Internal(String),
}

impl std::error::Error for UiError { }

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::_WindowCreation(msg) => write!(f, "Window creation error: {}", msg),
            Self::_WindowOperation(msg) => write!(f, "Window operation error: {}", msg),
            Self::Platform(e) => write!(f, "Platform error: {}", e),
            Self::_Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<PlatformError> for UiError {
    fn from(e: PlatformError) -> Self {
        Self::Platform(e)
    }
}

type _UiResult<T> = Result<T, UiError>;