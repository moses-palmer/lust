use crate::Value;

use super::cmd::Command;

/// An environment in which a script runs.
#[derive(Debug)]
pub enum Environment<'a, 'b, C>
where
    C: Command,
{
    /// An empty environment.
    Empty,

    /// A borrowed environment.
    Borrowed {
        /// The values.
        values: &'b [Value<'a, C::Tag>],
    },

    /// A scoped environment inheriting from another one.
    Scoped {
        /// The parent scope.
        parent: &'b Environment<'a, 'b, C>,

        /// The values.
        values: &'b [Value<'a, C::Tag>],
    },
}

impl<'a, 'b, C> Environment<'a, 'b, C>
where
    C: Command,
{
    /// An empty environment.
    pub const EMPTY: Self = Self::Empty;

    /// An empty environment.
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Creates an initial environment with borrowed names and values.
    ///
    /// # Arguments
    /// *  `scope` - The inner scope.
    pub fn borrowed(values: &'b [Value<'a, C::Tag>]) -> Self {
        Self::Borrowed { values }
    }

    /// Creates an environment inheriting from this one.
    ///
    /// # Arguments
    /// *  `scope` - The inner scope.
    pub fn with_scope(&'b self, values: &'b [Value<'a, C::Tag>]) -> Self {
        Self::Scoped {
            parent: self,
            values,
        }
    }

    /// Attempts to resolve a name.
    ///
    /// The value `0` corresponds to the last item in this environment. If the value is greater
    /// than the length of the current environment, the parent, if any, is queried with `key -
    /// length_of_current_environment`
    ///
    /// # Arguments
    /// *  `key` - The key to resolve.
    pub fn resolve(&self, key: usize) -> Option<Value<'a, C::Tag>> {
        use Environment::*;
        match self {
            Empty => None,
            Borrowed { values } => {
                if key < values.len() {
                    Some(values[values.len() - key - 1])
                } else {
                    None
                }
            }

            Scoped { parent, values } => {
                if key < values.len() {
                    Some(values[values.len() - key - 1])
                } else {
                    parent.resolve(key - values.len())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Environment;

    use crate::test_helpers::*;

    #[test]
    fn empty() {
        // Arrange
        let tested = Environment::<Command>::empty();

        // Act and assert
        assert!(matches!(tested, Environment::Empty));
    }

    #[test]
    fn borrowed() {
        // Arrange
        let values = [Value::Number(1.0), Value::Number(2.0)];
        let tested = Environment::<Command>::borrowed(&values);

        // Act and assert
        assert!(matches!(tested, Environment::Borrowed { .. }));
    }

    #[test]
    fn with_scope() {
        // Arrange
        let values = [Value::Number(1.0), Value::Number(2.0)];
        let t = Environment::<Command>::empty();
        let tested = t.with_scope(&values);

        // Act and assert
        assert!(matches!(tested, Environment::Scoped { .. }));
    }

    #[test]
    fn resolve_empty() {
        // Arrange
        let tested = Environment::<Command>::empty();
        let expected = None;

        // Act
        let actual = tested.resolve(2);

        // Act and assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn resolve_borrowed() {
        // Arrange
        let values = [Value::Number(1.0), Value::Number(2.0)];
        let tested = Environment::<Command>::borrowed(&values);
        let expected = Some(values[0]);

        // Act
        let actual = tested.resolve(1);

        // Act and assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn resolve_single() {
        // Arrange
        let values = [Value::Number(1.0), Value::Number(2.0)];
        let t = Environment::<Command>::empty();
        let tested = t.with_scope(&values);
        let expected = Some(values[0]);

        // Act
        let actual = tested.resolve(1);

        // Act and assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn resolve_multi() {
        // Arrange
        let values1 = [Value::Number(1.0)];
        let values2 = [Value::Number(2.0)];
        let t = Environment::<Command>::empty();
        let t = t.with_scope(&values1);
        let tested = t.with_scope(&values2);
        let expected1 = Some(values1[0]);
        let expected2 = Some(values2[0]);

        // Act
        let actual1 = tested.resolve(1);
        let actual2 = tested.resolve(0);

        // Act and assert
        assert_eq!(actual1, expected1);
        assert_eq!(actual2, expected2);
    }
}
