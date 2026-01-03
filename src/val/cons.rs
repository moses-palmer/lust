use std::fmt::Display;

/// The tail of a [`Cons`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Cdr<'a, T> {
    /// An empty list.
    #[default]
    Empty,

    /// The tail is a cons.
    Cons(&'a Cons<'a, T>),
}

impl<'a, T> Cdr<'a, T>
where
    T: Copy,
{
    /// The next cons, if any.
    pub fn next(&self) -> Option<&'a Cons<'a, T>> {
        match self {
            Cdr::Empty => None,
            Cdr::Cons(cons) => Some(cons),
        }
    }

    /// Iterates over the values of this list.
    pub fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        CdrIter(self.next().map(Cons::cons_iter))
    }
}

impl<'a, T> Display for Cdr<'a, T>
where
    T: Copy + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::common::write_list(self.iter(), f)
    }
}

struct CdrIter<'a, T>(Option<ConsIter<'a, T>>)
where
    T: Copy;

impl<'a, T> Iterator for CdrIter<'a, T>
where
    T: Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|iter| iter.next())
    }
}

/// A linked list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cons<'a, T> {
    /// The head of the list.
    car: T,

    /// The tail of the list.
    cdr: Cdr<'a, T>,
}

impl<'a, T> Cons<'a, T>
where
    T: Copy,
{
    /// Constructs a cons with a single value.
    ///
    /// # Arguments
    /// *  `car` - The head value.
    pub fn single(car: T) -> Self {
        Self {
            car,
            cdr: Cdr::Empty,
        }
    }

    /// Prepends a value to this cons and returns the new head.
    ///
    /// # Arguments
    /// *  `car` - The value.
    pub fn prepend(&'a self, car: T) -> Self {
        Self {
            car,
            cdr: Cdr::Cons(self),
        }
    }

    /// The tail of this list.
    ///
    /// This may be `self` if `self` is the last cell.
    pub fn tail(&self) -> &Self {
        let mut tail = self;
        while let Cdr::Cons(next) = tail.cdr {
            tail = next;
        }
        tail
    }

    /// The head of the list.
    pub fn car(&self) -> &T {
        &self.car
    }

    /// The tail of the list.
    pub fn cdr(&self) -> &Cdr<'a, T> {
        &self.cdr
    }

    /// Iterates over the values of this cons.
    #[inline]
    pub fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        self.cons_iter()
    }

    /// The iterator as a specific type.
    #[inline]
    fn cons_iter(&'a self) -> ConsIter<'a, T> {
        ConsIter(Some(self))
    }
}

impl<'a, T> Display for Cons<'a, T>
where
    T: Copy + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::common::write_list(self.iter(), f)
    }
}

struct ConsIter<'a, T>(Option<&'a Cons<'a, T>>)
where
    T: Copy;

impl<'a, T> Iterator for ConsIter<'a, T>
where
    T: Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let cons = self.0?;
        let value = &cons.car;
        self.0 = match cons.cdr {
            Cdr::Empty => None,
            Cdr::Cons(cons) => Some(cons),
        };
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdr_cons_empty() {
        // Arrange
        let expected = None;
        let tested = Cdr::<()>::Empty;

        // Act
        let actual = tested.next();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cdr_cons_some() {
        // Arrange
        let expected = Cons {
            car: 42,
            cdr: Cdr::Empty,
        };
        let tested = Cdr::Cons(&expected);

        // Act
        let actual = tested.next();

        // Assert
        assert_eq!(actual, Some(&expected));
    }

    #[test]
    fn cdr_iter_empty() {
        // Arrange
        let expected = Vec::<()>::new();
        let tested = Cdr::<()>::Empty;

        // Act
        let actual = tested.iter().cloned().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cdr_iter_some() {
        // Arrange
        let expected = vec![1, 5, 14];
        let tested = Cdr::Cons(&Cons {
            car: 1,
            cdr: Cdr::Cons(&Cons {
                car: 5,
                cdr: Cdr::Cons(&Cons {
                    car: 14,
                    cdr: Cdr::Empty,
                }),
            }),
        });

        // Act
        let actual = tested.iter().cloned().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cdr_fmt_empty() {
        // Arrange
        let expected = "()".to_string();
        let tested = Cdr::<i32>::Empty;

        // Act
        let actual = tested.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cdr_fmt_some() {
        // Arrange
        let expected = "(1 5 14)".to_string();
        let tested = Cdr::Cons(&Cons {
            car: 1,
            cdr: Cdr::Cons(&Cons {
                car: 5,
                cdr: Cdr::Cons(&Cons {
                    car: 14,
                    cdr: Cdr::Empty,
                }),
            }),
        });

        // Act
        let actual = tested.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cons_single() {
        // Arrange
        let expected = Cons {
            car: 42,
            cdr: Cdr::Empty,
        };

        // Act
        let actual = Cons::single(42);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cons_prepend() {
        // Arrange
        let expected = Cons {
            car: 1,
            cdr: Cdr::Cons(&Cons {
                car: 5,
                cdr: Cdr::Cons(&Cons {
                    car: 14,
                    cdr: Cdr::Empty,
                }),
            }),
        };

        // Act
        let t = Cons::single(14);
        let t = t.prepend(5);
        let actual = t.prepend(1);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cons_tail() {
        // Arrange
        let expected = Cons {
            car: 14,
            cdr: Cdr::Empty,
        };

        // Act
        let t = Cons {
            car: 1,
            cdr: Cdr::Cons(&Cons {
                car: 5,
                cdr: Cdr::Cons(&Cons {
                    car: 14,
                    cdr: Cdr::Empty,
                }),
            }),
        };
        let actual = t.tail();

        // Assert
        assert_eq!(actual, &expected);
    }

    #[test]
    fn cons_iter() {
        // Arrange
        let expected = vec![1, 5, 14];

        // Act
        let actual = Cons {
            car: 1,
            cdr: Cdr::Cons(&Cons {
                car: 5,
                cdr: Cdr::Cons(&Cons {
                    car: 14,
                    cdr: Cdr::Empty,
                }),
            }),
        }
        .iter()
        .cloned()
        .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }
}
