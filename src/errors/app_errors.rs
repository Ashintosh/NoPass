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


#[derive(Debug)]
enum _AppError {
    Other {
        message: String,
        file: &'static str,
        line: u32,
    },
}

impl _AppError {
    fn _generate(&self) {
        let _user_msg = match self {
            Self::Other { message, file, line } => {
                error!("Error at {}:{} - {}", file, line, message);
                message.clone()
            },
        };
    }
}