//! This module concerns itself with rust types and how they interact
//! with graphql types.

mod alignment;
mod parsing;
mod validation;

// TODO: This should not be pub, need to find a better way to do it...
pub mod parsing2;

pub use self::{
    alignment::align_output_type,
    parsing::{parse_rust_type, RustType},
    validation::{
        check_input_types_are_compatible, check_spread_type, check_types_are_compatible,
        outer_type_is_option, CheckMode,
    },
};
