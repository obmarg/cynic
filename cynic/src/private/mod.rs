#![doc(hidden)]
//! This module contains private code used by the derives.
//!
//! The API in here is absolutely unstable and should not be used by user code.

mod content;
mod flatten_de;
mod inline_fragment_de;
mod key_de;
mod spread_de;

pub use flatten_de::Flattened;
pub use inline_fragment_de::InlineFragmentVisitor;
pub use spread_de::Spreadable;
