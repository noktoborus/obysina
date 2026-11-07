#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod detail;
mod error;
mod item;
mod keys;
mod notice;
mod search;
mod state;
mod view;

pub use detail::Detail;
pub use error::{MenuError, Result};
pub use item::MenuItem;
pub use keys::{KEYS, Walk};
pub use notice::Notice;
pub use state::{MenuState, Row};
pub use view::{Beside, Chosen, PlateMenu};
