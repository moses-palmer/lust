// Tagged values must implement a few traits
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Tag {}
impl crate::val::Tag for Tag {}

struct Context;

commands_all! {
    /// # Scripting language reference
    pub enum ScriptReference<
        Tag = Tag,
        Context = Context,
    > {}
}
