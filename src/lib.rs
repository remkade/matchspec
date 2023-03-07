#![doc = include_str ! ("../README.md")]

pub mod matchspec;
mod input_table;
mod parsers;
pub mod package_candidate;
pub mod error;
#[cfg(feature = "python")]
pub mod python;

pub use crate::matchspec::*;
