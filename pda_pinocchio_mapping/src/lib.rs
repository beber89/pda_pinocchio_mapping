#![no_std]
#![deny(missing_docs)]
//! On-chain utility for Pinocchio programs.
//!
//! Example:
//! ```ignore
//! use pda_pinocchio_mapping::{mapping, Mapping, Bumpy};
//! ```

mod macros;
mod pinocchio_mapping;
pub use pinocchio_mapping::{Bumpy, Mapping};
