use std::str::FromStr;

/// Allows to returns a string that can be used by [FromStr]
pub trait ToParseableString {
    fn to_parseable_string(&self) -> String;
}

impl ToParseableString for String {
    fn to_parseable_string(&self) -> String {
        self.into()
    }
}