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
        /// The value names.
        names: &'b [&'b str],

        /// The values.
        values: &'b [Value<'a, C::Tag>],
    },

    /// A scoped environment inheriting from another one.
    Scoped {
        /// The parent scope.
        parent: &'b Environment<'a, 'b, C>,

        /// The value names.
        names: &'b [String],

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
    pub fn borrowed(names: &'b [&'b str], values: &'b [Value<'a, C::Tag>]) -> Self {
        Self::Borrowed { names, values }
    }

    /// Creates an environment inheriting from this one.
    ///
    /// # Arguments
    /// *  `scope` - The inner scope.
    pub fn with_scope(&'b self, names: &'b [String], values: &'b [Value<'a, C::Tag>]) -> Self {
        Self::Scoped {
            parent: self,
            names,
            values,
        }
    }

    /// Attempts to resolve a name.
    ///
    /// # Arguments
    /// *  `key` - The name to resolve.
    pub fn resolve(&self, key: &str) -> Option<Value<'a, C::Tag>> {
        use Environment::*;
        match self {
            Empty => None,
            Borrowed { names, values } => names
                .iter()
                .zip(values.iter())
                .find_map(|(k, v)| if *k == key { Some(*v) } else { None }),
            Scoped {
                parent,
                names,
                values,
            } => names
                .iter()
                .zip(values.iter())
                .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
                .or_else(|| parent.resolve(key)),
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
        let keys = ["a", "b"];
        let tested = Environment::<Command>::borrowed(&keys, &values);

        // Act and assert
        assert!(matches!(tested, Environment::Borrowed { .. }));
    }

    #[test]
    fn with_scope() {
        // Arrange
        let values = [Value::Number(1.0), Value::Number(2.0)];
        let keys = ["a".into(), "b".into()];
        let t = Environment::<Command>::empty();
        let tested = t.with_scope(&keys, &values);

        // Act and assert
        assert!(matches!(tested, Environment::Scoped { .. }));
    }

    #[test]
    fn resolve_empty() {
        // Arrange
        let tested = Environment::<Command>::empty();
        let expected = None;

        // Act
        let actual = tested.resolve("key");

        // Act and assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn resolve_borrowed() {
        // Arrange
        let values = [Value::Number(1.0), Value::Number(2.0)];
        let keys = ["a", "b"];
        let tested = Environment::<Command>::borrowed(&keys, &values);
        let expected = Some(values[0]);

        // Act
        let actual = tested.resolve("a");

        // Act and assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn resolve_single() {
        // Arrange
        let values = [Value::Number(1.0), Value::Number(2.0)];
        let keys = ["a".into(), "b".into()];
        let t = Environment::<Command>::empty();
        let tested = t.with_scope(&keys, &values);
        let expected = Some(values[0]);

        // Act
        let actual = tested.resolve("a");

        // Act and assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn resolve_multi() {
        // Arrange
        let keys1 = ["a".into()];
        let values1 = [Value::Number(1.0)];
        let keys2 = ["b".into()];
        let values2 = [Value::Number(2.0)];
        let t = Environment::<Command>::empty();
        let t = t.with_scope(&keys1, &values1);
        let tested = t.with_scope(&keys2, &values2);
        let expected1 = Some(values1[0]);
        let expected2 = Some(values2[0]);

        // Act
        let actual1 = tested.resolve("a");
        let actual2 = tested.resolve("b");

        // Act and assert
        assert_eq!(actual1, expected1);
        assert_eq!(actual2, expected2);
    }
}
