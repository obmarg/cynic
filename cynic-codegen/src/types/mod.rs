//! This module concerns itself with rust types and how they interact
//! with graphql types.

mod parsing;
mod validation;

pub use parsing::{parse_rust_type, RustType};

pub use self::validation::{
    check_input_types_are_compatible, check_spread_type, check_types_are_compatible,
    outer_type_is_option, CheckMode,
};
