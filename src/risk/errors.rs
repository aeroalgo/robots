use thiserror::Error;

#[derive(Debug, Error)]
pub enum StopHandlerError {
    #[error("unknown stop handler: {0}")]
    UnknownHandler(String),
    #[error("invalid parameter {0}")]
    InvalidParameter(String),
}

#[derive(Debug, Error)]
pub enum TakeHandlerError {
    #[error("unknown take handler: {0}")]
    UnknownHandler(String),
    #[error("invalid parameter {0}")]
    InvalidParameter(String),
}


