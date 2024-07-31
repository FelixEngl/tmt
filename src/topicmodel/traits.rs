/// Allows to returns a string that can be used by [FromStr]
pub trait ToParseableString {
    /// This string is guaranteed to be parseable, no matter that is represents.
    fn to_parseable_string(&self) -> String;
}

impl ToParseableString for String {
    fn to_parseable_string(&self) -> String {
        self.into()
    }
}