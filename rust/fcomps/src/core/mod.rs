mod base;
mod composit;
pub mod convert;
pub mod validate;

pub use base::*;
pub use composit::*;
pub use validate::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FcompError {
    #[error("Fail in convert from: {from} to: {to}")]
    ConvertType { from: String, to: String },
}
