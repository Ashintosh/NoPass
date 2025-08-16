/////////////////////////////////////////////////////////////////
///  NOTE:
///  - Custom error handling (UiError and UiResult) is defined
///    but not fully integrated throughout the code yet.
///  - Future improvements will focus on robust error handling,
///    graceful recovery, and better logging.
/// 
/// Please treat this as a work-in-progress
/////////////////////////////////////////////////////////////////

use log::error;
use thiserror::Error;


#[derive(Error, Debug)]
pub(crate) enum UiError {
    #[error("UI platform error occurred: {message}\nsource: {source}")]
    PlatformError {
        source: slint::PlatformError,
        message: String,
    },

    #[error("UI error: {0}")]
    Generic(String),
}

#[macro_export]
macro_rules! ui_error {
    (PlatformError, $source:expr, $msg:expr) => {
        UiError::PlatformError {
            source: $source,
            message: $msg.into(),
        }
    };

    (Generic, $msg:expr) => {
        UiError::Generic($msg.into())
    };
}