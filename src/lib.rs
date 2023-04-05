#![doc = include_str ! ("../README.md")]

pub mod error;
mod input_table;
pub mod matchspec;
pub mod package_candidate;
mod parsers;
pub mod python;

pub use crate::matchspec::*;
