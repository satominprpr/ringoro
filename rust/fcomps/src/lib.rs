pub mod behavior;
pub mod core;
pub mod macros;
pub mod service;

pub use self::core::validate::{Deny, Identity, Through};
pub use self::core::*;
use ringoro_utils::*;

pub trait Functor {
    type Result;
}
