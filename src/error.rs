use actix_web::http::StatusCode;
use actix_web::ResponseError;
use paperclip::actix::api_v2_errors;
use thiserror::Error;
use crate::PlaybookFunction;

pub type Result<T> = std::result::Result<T, HttpError>;

#[derive(Debug, Error)]
#[api_v2_errors(
    code=400,
    code=401,
    code=500,
)]
pub enum HttpError {
    #[error("An internal error occurred")]
    Mysql(#[from] mysql::Error),
    #[error("Unauthorized")]
    Unauthorized
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Mysql(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("MySQL error: {0:?}")]
    Mysql(#[from] mysql::Error),
    #[error("Invalid IP Address: {0:?}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("Unsupported: {0}")]
    Unsupported(&'static str),
    #[error("Unable to parse Integer: {0:?}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Missing playbook proving function {0:?}")]
    MissingPlaybook(PlaybookFunction),
    #[error("IO Error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Playbook did not exit successfully")]
    UnsuccessfulPlaybook
}