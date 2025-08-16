/////////////////////////////////////////////////////////////////
///  NOTE:
///  - Custom error handling (AppError) is defined
///    but not fully integrated throughout the code yet.
///  - Future improvements will focus on robust error handling,
///    graceful recovery, and better logging.
/// 
/// Please treat this as a work-in-progress
/////////////////////////////////////////////////////////////////

use log::error;
use thiserror::Error;


#[derive(Error, Debug)]
pub(crate) enum AppError {
    #[error("{msg}: {source}")]
    IoError {
        msg: String,
        #[source]
        source: std::io::Error,
    },

    #[error("{msg}: {str_source}")]
    PoisedState {
        msg: String,
        str_source: String,
    },

    #[error("App error: {0}")]
    Generic(String),
}

#[macro_export]
macro_rules! app_error {
    (IoError, $err:expr, $msg:expr) => {
        AppError::IoError {
            msg: $msg.into(),
            source: $err,
        }
    };

    (PoisedState, $err:expr, $msg:expr) => {
        AppError::PoisedState {
            msg: $msg.into(),
            str_source: $err,
        }
    };

    (Generic, $msg:expr) => {
        AppError::Generic($msg.into())
    };
}

