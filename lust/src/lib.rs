#![doc = include_str!("../../README.md")]
#![cfg_attr(
    not(feature = "doc"),
    doc = "Please build the documentation with the feature `doc` to generate a reference.\n"
)]
pub use lust_macros::*;

#[cfg(feature = "doc")]
mod doc;

#[cfg(feature = "doc")]
#[doc(inline)]
pub use doc::ScriptReference;
