//! # A tiny interpreter for a LISP like language
//!
//! This crate provides a scripting environment.
//!
#![cfg_attr(
    not(feature = "doc"),
    doc = "Please build the documentation with the feature `doc` to generate a reference.\n"
)]
#![cfg_attr(
    feature = "doc",
    doc = "Please see [here](crate::doc::ScriptReference) for a reference of the language.\n"
)]

#[macro_use]
mod macros;

#[macro_use]
pub mod eval;

pub mod ast;
pub mod exp;
pub mod lambda;
pub mod val;

#[cfg(feature = "doc")]
pub mod doc;

#[cfg(test)]
pub(crate) mod test_helpers;

pub(crate) mod common;
pub use common::Serializable;

pub use exp::{Expression, cmd::Command, env::Environment, linked::Script};
pub use val::{Value, Values};
