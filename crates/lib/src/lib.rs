//! # A tiny interpreter for a LISP like language
//!
//! This crate provides a scripting environment.

pub mod alloc;
pub mod ast;
pub mod exp;
pub mod lambda;
pub mod script;
pub mod val;

#[cfg(test)]
pub(crate) mod test_helpers;

pub(crate) mod common;
pub use common::Serializable;

pub use alloc::Allocator;
pub use exp::{
    Expression,
    cmd::{Command, Context},
    env::Environment,
};
pub use script::Script;
pub use val::{Value, Values, cons::Cons};
