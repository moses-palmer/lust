use std::{cell::RefCell, mem::MaybeUninit};

use super::Error;

/// A bounded allocator.
///
/// This allocator can allocate up to `N` items, and then every allocation attempt will fail.
pub struct Allocator<const N: usize, T> {
    /// The index of the next slot.
    next_index: RefCell<usize>,

    /// The items.
    items: RefCell<[MaybeUninit<T>; N]>,
}

impl<const N: usize, T> Default for Allocator<N, T> {
    fn default() -> Self {
        Self {
            next_index: RefCell::new(0),
            items: RefCell::new([const { MaybeUninit::uninit() }; N]),
        }
    }
}

impl<const N: usize, T> Drop for Allocator<N, T> {
    fn drop(&mut self) {
        let end = *self.next_index.borrow();
        for i in (0..end).rev() {
            // SAFETY: We only increment the index of the next slot after successful insertion, so
            // all values up to end exclusive are initialised
            unsafe {
                self.items.borrow_mut()[i].assume_init_drop();
            }
        }
    }
}

impl<'a, const N: usize, T> super::Allocator<'a> for Allocator<N, T>
where
    T: Sized + 'a,
{
    type Item = T;

    fn alloc(&self, value: T) -> Result<&'a Self::Item, Error> {
        // Index is the index of the slot into which the value should be put; continue only if this
        // is inside the bucket
        let index = *self.next_index.borrow();
        if index < N {
            let mut items = self.items.borrow_mut();

            // SAFETY: We write the value before reading it
            items[index].write(value);
            let result = unsafe { items[index].as_ptr().as_ref().unwrap() };

            *self.next_index.borrow_mut() += 1;
            Ok(result)
        } else {
            Err(Error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::alloc::Allocator;

    #[test]
    fn alloc() {
        // Arrange
        let tested = super::Allocator::<10, u32>::default();

        // Act
        let a = tested.alloc(42);
        let b = tested.alloc(7623);

        // Assert
        assert_eq!(Ok(&42), a);
        assert_eq!(Ok(&7623), b);
    }

    #[test]
    fn alloc_exhausted() {
        // Arrange
        let tested = super::Allocator::<1, u32>::default();

        // Act
        let a = tested.alloc(42);
        let b = tested.alloc(7623);

        // Assert
        assert_eq!(Ok(&42), a);
        assert_eq!(Err(Error), b);
    }
}
