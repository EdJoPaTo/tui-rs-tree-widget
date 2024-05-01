/// Selector which can be used as `Identifier` for common Key Value structures like JSON.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum Selector {
    ObjectKey(String),
    ArrayIndex(usize),
    #[default]
    None,
}

impl core::fmt::Display for Selector {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ObjectKey(key) => key.fmt(fmt),
            Self::ArrayIndex(index) => index.fmt(fmt),
            Self::None => Ok(()),
        }
    }
}

#[test]
fn display_object() {
    let selector = Selector::ObjectKey("foo".to_owned());
    let result = format!("{selector}");
    assert_eq!(result, "foo");
}

#[test]
fn display_array() {
    let selector = Selector::ArrayIndex(42);
    let result = format!("{selector}");
    assert_eq!(result, "42");
}

#[test]
fn display_none() {
    let selector = Selector::None;
    let result = format!("{selector}");
    assert_eq!(result, "");
}
