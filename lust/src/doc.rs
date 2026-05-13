#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[doc(hidden)]
pub enum Tag {}
crate::tag!(Tag);

#[doc(hidden)]
pub struct Context;
impl crate::Context for Context {}

crate::commands_all! {
    /// # Scripting language reference.
    #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    pub enum ScriptReference<
        Tag = Tag,
        Context = Context,
    > {}
}
