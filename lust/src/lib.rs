#![cfg_attr(
    not(feature = "doc"),
    doc = "Please build the documentation with the feature `doc` to generate a reference.\n"
)]
#![cfg_attr(
    feature = "doc",
    doc = "Please see [here](crate::doc::ScriptReference) for a reference of the language.\n"
)]
pub use lust_macros::*;

#[cfg(feature = "doc")]
pub mod doc;
