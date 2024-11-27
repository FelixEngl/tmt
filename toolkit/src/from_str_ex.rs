use std::str::FromStr;
use sealed::sealed;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{}Failed parsing of {parse_target:?}.\nCause: {source}", .tag.map(|v| format!("{v}: ")).unwrap_or_else(|| "".to_string()))]
pub struct ParseErrorEx<E> {
    parse_target: String,
    tag: Option<&'static str>,
    #[source]
    source: E
}

impl<E> ParseErrorEx<E> {
    pub fn new(parse_target: String, tag: Option<&'static str>, source: E) -> Self {
        Self { parse_target, tag, source }
    }

    pub fn new_from_str(parse_target: &str, tag: Option<&'static str>, source: E) -> Self {
        Self::new(parse_target.to_string(), tag, source)
    }

    pub fn without_tag(parse_target: &str, source: E) -> Self {
        Self::new_from_str(parse_target, None, source)
    }

    pub fn with_tag(parse_target: &str, tag: &'static str, source: E) -> Self {
        Self::new_from_str(parse_target, Some(tag), source)
    }
}

pub trait FromStrEx: FromStr {
    fn from_str_ex(s: &str) -> Result<Self, ParseErrorEx<<Self as FromStr>::Err>>;
    fn from_str_ex_tagged(s: &str, tag: &'static str) -> Result<Self, ParseErrorEx<<Self as FromStr>::Err>>;
}

impl<T> FromStrEx for T where T: FromStr {
    fn from_str_ex(s: &str) -> Result<Self, ParseErrorEx<<Self as FromStr>::Err>> {
        <Self as FromStr>::from_str(s).map_err(|e| {
            ParseErrorEx::without_tag(s, e)
        })
    }

    fn from_str_ex_tagged(s: &str, tag: &'static str) -> Result<Self, ParseErrorEx<<Self as FromStr>::Err>> {
        <Self as FromStr>::from_str(s).map_err(|e| {
            ParseErrorEx::with_tag(s, tag, e)
        })
    }
}

#[sealed]
pub trait ParseEx {
    fn parse_ex<F: FromStr>(&self) -> Result<F, ParseErrorEx<F::Err>>;
    fn parse_ex_tagged<F: FromStr>(&self, tag: &'static str) -> Result<F, ParseErrorEx<F::Err>>;
}

#[sealed]
impl ParseEx for str {
    fn parse_ex<F: FromStr>(&self) -> Result<F, ParseErrorEx<F::Err>> {
        F::from_str_ex(self)
    }

    fn parse_ex_tagged<F: FromStr>(&self, tag: &'static str) -> Result<F, ParseErrorEx<F::Err>> {
        F::from_str_ex_tagged(self, tag)
    }
}