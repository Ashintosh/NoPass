use log::error;
use slint::PlatformError;

#[derive(Debug)]
enum AppError {
    Other {
        message: String,
        file: &'static str,
        line: u32,
    },
}

impl AppError {
    fn generate(&self) {
        let user_msg = match self {
            Self::Other { message, file, line } => {
                error!("Error at {}:{} - {}", file, line, message);
                message.clone()
            },
        };

        

        
    }
}