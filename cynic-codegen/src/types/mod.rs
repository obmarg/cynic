//! This module concerns itself with rust types and how they interact
//! with graphql types.

mod alignment;
mod parsing;
mod validation;

pub use self::{
    alignment::{align_input_type, align_output_type},
    validation::{
        check_input_types_are_compatible, check_spread_type, check_types_are_compatible,
        outer_type_is_option, CheckMode,
    },
};
