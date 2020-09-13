//! Provides Cynic implementations for some external types

#[cfg(feature = "chrono")]
/// Cynic support for [chrono](https://github.com/chronotope/chrono) types.
pub mod chrono;

#[cfg(feature = "bson")]
/// Cynic support for [bson](https://github.com/mongodb/bson-rust) types.
pub mod bson;
