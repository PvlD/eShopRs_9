use tokio::sync::mpsc::error::SendError;

#[derive(Debug)]
pub enum AppError {
    Json(serde_json::Error),
    OtherError(String),
    ContentProcessorError(String),
    SendError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::OtherError(e) => write!(f, "Other error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::ContentProcessorError(e) => write!(f, "Content processor error: {}", e),
            AppError::SendError(e) => write!(f, "Send error: {}", e),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}

impl<T> From<SendError<T>> for AppError {
    fn from(e: SendError<T>) -> Self {
        AppError::SendError(e.to_string())
    }
}
