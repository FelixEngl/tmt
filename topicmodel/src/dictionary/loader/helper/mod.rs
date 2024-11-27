mod xml_reader;
pub(super) mod gen_freedict_tei_reader;
pub(super) mod gen_iate_tbx_reader;
pub(super) mod gen_ms_terms_reader;

use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom};
use nom::error::{ParseError};
use nom::{AsChar, InputTakeAtPosition, IResult, Slice, InputIter, InputLength};
use nom::combinator::{map};

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct OptionalEntry<T>(pub Option<T>);

impl<T> OptionalEntry<T> {
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T> From<Option<T>> for OptionalEntry<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> Display for OptionalEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => {
                write!(f, "-NONE-")
            }
            Some(ref value) => {
                write!(f, "{value}")
            }
        }
    }
}


macro_rules! take_bracket {
    ($open: literal, $close: literal) => {
        nom::sequence::delimited(
            nom::character::complete::char($open),
            nom::bytes::complete::take_until1(const_str::to_str!($close)),
            nom::character::complete::char($close)
        )
    };
}

pub fn take_nested_bracket_delimited<I, E: ParseError<I>>(
    opening_bracket: char,
    closing_bracket: char,
) -> impl FnMut(I) -> IResult<I, I, E>
where I: Slice<RangeFrom<usize>> + Slice<Range<usize>> + InputIter + InputLength,
      <I as InputIter>::Item: AsChar,
{
    map(
        take_nested_bracket(opening_bracket, closing_bracket),
        |value: I| value.slice(1..value.input_len() - 1)
    )
}

// Base from: https://stackoverflow.com/questions/70630556/parse-allowing-nested-parentheses-in-nom
pub fn take_nested_bracket<I, E: ParseError<I>>(
    opening_bracket: char,
    closing_bracket: char,
) -> impl Fn(I) -> IResult<I, I, E>
where I: Slice<RangeFrom<usize>> + Slice<Range<usize>> + InputIter,
      <I as InputIter>::Item: AsChar,
{
    move |i: I| {
        match i
            .iter_elements()
            .next()
            .map(|t| {
                t.as_char() == opening_bracket
            })
        {
            Some(true) => {}
            _ => return Err(nom::Err::Error(E::from_char(i, opening_bracket)))
        }
        let mut index = 0;
        let mut bracket_counter = 0;
        while let Some(n) = i.slice(index..).position(
            |value|  {
                let c = value.as_char();
                c == opening_bracket || c == closing_bracket
            }
        ) {
            index += n;
            let mut it = i.slice(index..).iter_elements();
            match it.next().unwrap().as_char() {
                c if c == opening_bracket => {
                    bracket_counter += 1;
                    index += opening_bracket.len_utf8();
                }
                c if c == closing_bracket => {
                    // Closing bracket.
                    bracket_counter -= 1;
                    index += closing_bracket.len_utf8();
                }
                // Can not happen.
                _ => unreachable!(),
            }
            match bracket_counter {
                0 => break,
                // We found the unmatched closing bracket.
                -1 => {
                    // We do not consume it.
                    index -= closing_bracket.len_utf8();
                    break
                }
                _ => {}
            }
        }

        if bracket_counter == 0 || bracket_counter == -1 {
            Ok((i.slice(index..), i.slice(0..index)))
        } else {
            Err(nom::Err::Error(E::from_char(i.slice(index..), closing_bracket)))
        }
    }
}


pub(crate) use take_bracket;

pub fn space_only0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
{
    input.split_at_position_complete(|item| {
        let c = item.as_char();
        !(c == ' ')
    })
}

pub trait HasLineInfo {
    fn current_buffer(&self) -> Option<&[u8]>;

    fn current_line_number(&self) -> usize;
}


#[cfg(test)]
pub mod test {
    use std::error::Error;
    use std::fmt::{Debug, Display};
    use nom::IResult;
    use crate::topicmodel::dictionary::loader::helper::{take_nested_bracket_delimited, HasLineInfo};

    pub fn execute_test_read_for<O, E, I, Iter>(
        function_based_line_wise_reader: I,
        err_max: usize,
        show_diff: usize
    )
    where
        O: Display + Debug,
        E: Error,
        I: IntoIterator<Item=Result<O, E>, IntoIter=Iter>,
        Iter: Iterator<Item=Result<O, E>> + HasLineInfo
    {
        let mut it = function_based_line_wise_reader.into_iter();

        let mut ct_err = 0usize;
        let mut ct_diff = 0usize;
        let mut ct_plain = 0usize;
        while let Some(v) = it.next() {
            if err_max != 0 && err_max == ct_err {
                break;
            }
            match v {
                Ok(value) => {
                    match it.current_buffer() {
                        None => {
                            ct_plain += 1;
                            if show_diff != 0 && ct_plain <= show_diff {
                                println!("Original:\n{}\n{:?}\n", value, value)
                            }

                        }
                        Some(buf) => {
                            match std::str::from_utf8(buf) {
                                Ok(s) => {
                                    if s.replace(' ', "").replace('\t', "").trim().ne(value.to_string().replace(' ', "").replace('\t', "").as_str().trim()) {
                                        ct_diff+=1;
                                        if show_diff != 0 && ct_diff <= show_diff {
                                            println!("Line: {}", it.current_line_number());
                                            println!("Original:\n{}\n{}\n{:?}\n", s.trim(), value, value)
                                        }
                                    }
                                }
                                Err(_) => {
                                    println!("Failed to parse original!")
                                }
                            }
                        }
                    }
                }
                Err(error) => {
                    ct_err += 1;
                    match it.current_buffer() {
                        Some(buf) => {
                            match std::str::from_utf8(buf) {
                                Ok(s) => {
                                    println!("Line: {}", it.current_line_number());
                                    println!("Original:\n{}", s.trim())
                                }
                                Err(_) => {
                                    println!("Failed to parse original!")
                                }
                            }
                        }
                        None => {}
                    }

                    println!("{:?}\n", error)
                }
            }
        }
        //23 82
        println!("Err: {ct_err}, Diff: {ct_diff}, Plain: {ct_plain}");

        assert_eq!(0, ct_err, "There were errors!");
    }

    #[test]
    fn can_find_correct_str(){
        const VALUES: &[&str] = &[
            "(from sth. / to sth.) and",
            "(from sth. / (in)to sth.) so",
            "(from sth. / (in)to sth.) so (with me) lul",
            "() we",
            "(how about dis?)",
            "(how about dis?",
            "too (how about dis?)",
        ];

        for value in VALUES.iter().copied() {
            let result: IResult<&str, &str, nom::error::VerboseError<&str>> = take_nested_bracket_delimited('(', ')')(value);
            println!("{result:?}");
        }

    }
}

