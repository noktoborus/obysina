#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod error;
mod kind;
mod store;

pub use error::{BackdropError, Result};
pub use kind::Kind;
pub use store::{Backdrop, Kept, Store};
