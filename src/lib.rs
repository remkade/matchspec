#![doc = include_str ! ("../README.md")]

pub mod matchspec;
mod input_table;
mod parsers;
pub mod package_candidate;

#[cfg(feature = "python")]
pub mod python;

