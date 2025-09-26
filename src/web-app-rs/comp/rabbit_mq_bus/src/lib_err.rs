#[derive(Debug)]
pub enum AppError {
    AMQPError(amqprs::error::Error),
    EBusError(ebus::lib_err::AppError),
    OtherError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::AMQPError(e) => write!(f, "AMQP error: {}", e),
            AppError::EBusError(e) => write!(f, "EBus error: {}", e),
            AppError::OtherError(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl From<amqprs::error::Error> for AppError {
    fn from(e: amqprs::error::Error) -> Self {
        AppError::AMQPError(e)
    }
}

impl From<ebus::lib_err::AppError> for AppError {
    fn from(e: ebus::lib_err::AppError) -> Self {
        AppError::EBusError(e)
    }
}
