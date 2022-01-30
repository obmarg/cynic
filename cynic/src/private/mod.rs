#![doc(hidden)]
//! This module contains private code used by the derives.
//!
//! The API in here is absolutely unstable and should not be used by user code.
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};

use self::content::Content;

mod content;
mod inline_fragment_de;
mod key_de;

pub use inline_fragment_de::InlineFragmentVisitor;
