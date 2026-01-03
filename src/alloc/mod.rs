use thiserror::Error;

pub mod bounded;
pub mod zero;

/// An error indicating that allocation has failed.
#[derive(Debug, Error, PartialEq)]
#[error("allocation failed")]
pub struct Error;

/// An allocator.
pub trait Allocator<'a>: Default {
    /// The type of items allocated.
    type Item: Sized + 'a;

    /// Allocates a single value.
    fn alloc(&self, value: Self::Item) -> Result<&'a Self::Item, Error>;
}
