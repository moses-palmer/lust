use super::Error;

/// A zero allocator.
///
/// This type cannot allocate any values.
pub struct Allocator<T>(::std::marker::PhantomData<T>);

impl<T> Default for Allocator<T> {
    fn default() -> Self {
        Self(::std::marker::PhantomData)
    }
}

impl<'a, T> super::Allocator<'a> for Allocator<T>
where
    T: 'a,
{
    type Item = T;

    fn alloc(&self, _value: T) -> Result<&'a Self::Item, Error> {
        Err(Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::alloc::Allocator;

    #[test]
    fn alloc_exhausted() {
        // Arrange
        let tested = super::Allocator::<u32>::default();

        // Act
        let a = tested.alloc(42);
        let b = tested.alloc(7623);

        // Assert
        assert_eq!(Err(Error), a);
        assert_eq!(Err(Error), b);
    }
}
