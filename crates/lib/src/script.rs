use crate::{
    Command, Cons, Context, Environment, Expression, Value, Values, alloc, ast, exp, lambda, val,
};

/// The context passed when evaluating commands.
pub struct EvaluationContext<'a, 'b, 'c, A, C>
where
    A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
    C: Command,
    'a: 'c,
    'b: 'c,
{
    /// The current script.
    pub script: &'a Script<C>,

    /// The current allocator.
    pub alloc: &'c A,

    /// The current script context.
    pub ctx: &'c C::Context,

    /// The current environment.
    pub env: &'c crate::Environment<'a, 'b, C>,
}

impl<'a, 'b, 'c, A, C> EvaluationContext<'a, 'b, 'c, A, C>
where
    A: alloc::Allocator<'a, Item = crate::Cons<'a, Value<'a, C::Tag>>> + 'a,
    C: Command,
    'a: 'c,
    'b: 'c,
{
    /// Evaluates an expression.
    ///
    /// # Arguments
    /// *  `e` - The expression to evaluate.
    pub fn value(&self, e: &'a Expression<C>) -> exp::Result<'a, C> {
        self.script.value(e, self.alloc, self.ctx, self.env)
    }

    /// Allocates a _cons_.
    ///
    /// # Argument
    /// *  `value` - The value to allocate.
    pub fn alloc(&self, value: A::Item) -> Result<&'a A::Item, alloc::Error> {
        self.alloc.alloc(value)
    }
}

/// A linked expression.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Script<C> {
    /// The root expression.
    root: super::Expression<C>,

    /// The lambda store
    lambdas: lambda::Store<C>,
}

impl<C> Script<C>
where
    C: Command,
{
    /// Constructs a script from an expression and a lambda store.
    ///
    /// # Arguments
    /// *  `root` - The root expression.
    /// *  `lambdas` - The lambda store.
    pub fn new(root: super::Expression<C>, lambdas: lambda::Store<C>) -> Self {
        Self { root, lambdas }
    }

    /// Evaluates the root expression given a context and environment.
    ///
    /// # Arguments
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The context of the evaluation.
    pub fn evaluate<'a, 'b, A>(
        &'a self,
        alloc: &A,
        ctx: &C::Context,
    ) -> Result<val::owned::Value<C::Tag>, exp::Error<'a>>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        self.value(&self.root, alloc, ctx, &Environment::empty())
            .and_then(|v| Ok(v.try_into()?))
    }

    /// Evaluates an expression given a context and environment.
    ///
    /// # Arguments
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The context of the evaluation.
    /// *  `env` - The current variable scope.
    pub fn value<'a, 'b, A>(
        &'a self,
        e: &'a super::Expression<C>,
        alloc: &A,
        ctx: &C::Context,
        env: &Environment<'a, 'b, C>,
    ) -> exp::Result<'a, C>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        ctx.on_evaluate()?;

        use super::Expression::*;
        match e {
            Void => Ok(Value::Void),
            List(v) => {
                let mut i = v.iter();
                if let Some(head) = i.next() {
                    let head = self.value(head, alloc, ctx, env)?;
                    if let Value::Lambda(lambda_ref) = head {
                        let arguments = i
                            .map(|e| self.value(e, alloc, ctx, env))
                            .collect::<Result<Values<_>, _>>()?;
                        self.invoke(alloc, ctx, lambda_ref, &arguments)
                            .unwrap_or_else(|| Err(val::Error::Operation("unknown lambda").into()))
                    } else {
                        Err(val::Error::Operation("cannot evaluate list").into())
                    }
                } else {
                    Ok(Value::NIL)
                }
            }
            Map(_, _) => Err(exp::Error::from(val::Error::Operation(
                "cannot evaluate map",
            ))),
            AST(v) => Ok(Value::AST(v)),
            Reference(v) => env.resolve(*v).ok_or_else(|| exp::Error::UnknownReference {
                value: format!("#{v}"),
            }),
            Boolean(v) => Ok((*v).into()),
            Number(v) => Ok((*v).into()),
            String(v) => Ok(v.as_str().into()),
            Command(v) => {
                ctx.on_evaluate()?;
                v.evaluate(self, alloc, ctx, env)
            }
            Lambda(v) => Ok((*v).into()),
        }
    }

    /// Evaluates a lambda.
    ///
    /// # Arguments
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The evaluation context.
    /// *  `lambda_ref` - A reference to the lambda to evaluate.
    /// *  `arguments` - The arguments to pass.
    pub fn invoke<'a, A>(
        &'a self,
        alloc: &A,
        ctx: &C::Context,
        lambda_ref: lambda::Ref,
        arguments: &[Value<'a, C::Tag>],
    ) -> Option<exp::Result<'a, C>>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        if let Some(lambda) = self.lambdas.resolve(lambda_ref) {
            Some(lambda.invoke(self, alloc, ctx, arguments))
        } else {
            None
        }
    }

    /// Calls a function for this and each sub-expression.
    ///
    /// # Argument
    /// *  `f` - The callback.
    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&super::Expression<C>),
    {
        self.root.for_each(f);
    }
}

impl<C> Default for Script<C> {
    fn default() -> Self {
        Self {
            root: Default::default(),
            lambdas: Default::default(),
        }
    }
}

impl<C> ::std::str::FromStr for Script<C>
where
    C: Command + ::std::fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut context = exp::ParseContext::default();
        let root = Expression::parse(
            &mut context,
            &ast::parse(&mut ast::tokenize(s)).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let lambdas = context.lambdas;
        Ok(Self { root, lambdas })
    }
}

impl<C> ::std::fmt::Display for Script<C>
where
    C: Command + ::std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl<'de, C> serde::Deserialize<'de> for Script<C>
where
    C: Command,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ScriptVisitor(Default::default()))
    }
}

/// A stand-alone function.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Function<C>
where
    C: Command,
{
    /// The main script.
    script: Script<C>,

    /// The main function.
    main: lambda::Ref,
}

impl<C> Function<C>
where
    C: Command,
{
    /// Invokes this function.
    ///
    /// # Arguments
    /// *  `alloc` - The allocator to use.
    /// *  `ctx` - The evaluation context.
    /// *  `lambda_ref` - A reference to the lambda to evaluate.
    /// *  `arguments` - The arguments to pass.
    pub fn invoke<'a, 'b, A>(
        &'a self,
        alloc: &A,
        ctx: &C::Context,
        arguments: &[Value<'a, C::Tag>],
    ) -> Result<val::owned::Value<C::Tag>, exp::Error<'a>>
    where
        A: alloc::Allocator<'a, Item = Cons<'a, Value<'a, C::Tag>>> + 'a,
        <C as Command>::Tag: 'a,
    {
        self.script
            .invoke(alloc, ctx, self.main, arguments)
            .ok_or(exp::Error::InvalidOperation(val::Error::Operation(
                "unknown lambda",
            )))
            .and_then(|v| Ok(v?.try_into()?))
    }
}

impl<C> TryFrom<Script<C>> for Function<C>
where
    C: Command,
{
    type Error = Script<C>;

    /// Attempts to convert a script to a function.
    ///
    /// This requires that the script evaluates to a lambda.
    fn try_from(script: Script<C>) -> Result<Self, Self::Error> {
        match script.root {
            Expression::Lambda(main) => Ok(Self { script, main }),
            _ => Err(script),
        }
    }
}

//#[cfg(feature = "serde")]
impl<'de, C> serde::Deserialize<'de> for Function<C>
where
    C: Command,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(ScriptVisitor(Default::default()))
            .and_then(|s| {
                Self::try_from(s).map_err(|_| serde::de::Error::custom("expected single lambda"))
            })
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Field {
    Root,
    Lambdas,
}

struct ScriptVisitor<C>(::std::marker::PhantomData<C>);

impl<'de, C> serde::de::Visitor<'de> for ScriptVisitor<C>
where
    C: Command,
{
    type Value = Script<C>;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("String")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(|e| E::custom(e))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        println!("Deserializing Script from map");

        let mut root = None;
        let mut lambdas = None;
        while let Some(key) = map.next_key::<Field>()? {
            match key {
                Field::Root => {
                    if root.replace(map.next_value()?).is_some() {
                        return Err(serde::de::Error::duplicate_field("root"));
                    }
                }
                Field::Lambdas => {
                    if lambdas.replace(map.next_value()?).is_some() {
                        return Err(serde::de::Error::duplicate_field("lambdas"));
                    }
                }
            }
        }

        match (root, lambdas) {
            (Some(root), Some(lambdas)) => Ok(Script { root, lambdas }),
            (Some(root), _) => Ok(Script {
                root,
                lambdas: lambda::Store::default(),
            }),
            _ => Err(serde::de::Error::missing_field("root")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_helpers::{Context, Tag};

    commands_all! {
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        pub enum Command<
            Tag = Tag,
            Context = Context,
        > {
            "debug" => Debug(_ctx, _a) {
                Ok(().into())
            }
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn parse_valid() {
        // Arrange
        let data = r#"(debug 42)"#;
        let expected = Script::<Command> {
            root: Expression::Command(Command::Debug(vec![Expression::Number(42.0)])),
            lambdas: lambda::Store::default(),
        };

        // Act
        let actual = data.parse::<Script<Command>>().unwrap();

        // Assert
        assert_eq!(actual.root, expected.root);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn parse_invalid() {
        // Arrange
        let data = r#"(debug 42"#;
        let expected = "unexpected end of tokens";

        // Act
        let actual = data.parse::<Script<Command>>().unwrap_err();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn deserialize_str() {
        // Arrange
        let data = r#""(debug 42)""#;
        let expected = Script::<Command> {
            root: Expression::Command(Command::Debug(vec![Expression::Number(42.0)])),
            lambdas: lambda::Store::default(),
        };

        // Act
        let actual = serde_json::from_str::<Script<Command>>(data).unwrap();

        // Assert
        assert_eq!(actual.root, expected.root);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn deserialize_struct() {
        // Arrange
        let data = r#"{
            "root": {
                "Command": {
                    "Debug": [
                        {
                            "Number": 42.0
                        }
                    ]
                }
            },
            "lambdas": {
                "data": []
            }
        }"#;
        let expected = Script::<Command> {
            root: Expression::Command(Command::Debug(vec![Expression::Number(42.0)])),
            lambdas: lambda::Store::default(),
        };

        // Act
        let actual = serde_json::from_str::<Script<Command>>(data).unwrap();

        // Assert
        assert_eq!(actual.root, expected.root);
    }

    #[test]
    fn function_from_lambda() {
        // Arrange
        let data = r#"(lambda (a b) (+ a b))"#;
        let script = data.parse::<Script<Command>>().unwrap();
        let tested = Function::try_from(script).unwrap();
        let expected = Ok(9.0.into());

        // Act
        let actual = tested.invoke(
            &alloc::bounded::Allocator::<32, _>::default(),
            &Context,
            &[4.0.into(), 5.0.into()],
        );

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn function_from_expression() {
        // Arrange
        let data = r#"(+ 1 2)"#;
        let script = data.parse::<Script<Command>>().unwrap();
        let expected = script.clone();

        // Act
        let actual = Function::try_from(script).unwrap_err();

        // Assert
        assert_eq!(actual.root, expected.root);
    }
}
