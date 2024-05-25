use nom::character::complete::multispace0;
use nom::character::is_alphanumeric;
use nom::error::ParseError;
use nom::Parser;
use nom::sequence::delimited;

/// Something surrounded by whitespaces
pub fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(
    f: F,
) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}

pub fn is_alphanum_or_underscore(c: u8) -> bool {
    is_alphanumeric(c) || c == b'_'
}