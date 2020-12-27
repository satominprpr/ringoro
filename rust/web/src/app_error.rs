use std::fmt;

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use log::error;

use crate::utils::result::{self, StdResult};

#[derive(Debug)]
pub struct AppError {
    error: result::Error,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<result::Error> for AppError {
    fn from(error: result::Error) -> Self {
        error!(target: "ringoro", "ERROR: {}", error);
        Self { error }
    }
}

pub type Responce<T = HttpResponse> = StdResult<T, AppError>;
