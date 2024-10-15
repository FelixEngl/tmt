macro_rules! take_bracket {
    ($open: literal, $close: literal) => {
        nom::sequence::delimited(
            nom::character::complete::char($open),
            nom::bytes::complete::take_until1(const_str::to_str!($close)),
            nom::character::complete::char($close)
        )
    };
}

use nom::error::ParseError;
use nom::{AsChar, InputTakeAtPosition, IResult};
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


pub fn merge_list<T, I: IntoIterator<Item=T>>(first: T, rest: I) -> Vec<T> {
    let mut consol = Vec::new();
    consol.push(first);
    consol.extend(rest);
    consol
}

pub fn merge_list_opt<T, I: IntoIterator<Item=T>>(first: T, rest: Option<I>) -> Vec<T> {
    match rest {
        None => {
            vec![first]
        }
        Some(value) => {
            merge_list(first, value)
        }
    }
}

#[inline(always)]
pub fn map_merge_list<T, I: IntoIterator<Item=T>>((first, rest): (T, I)) -> Vec<T> {
    merge_list(first, rest)
}

#[inline(always)]
pub fn map_merge_list_opt<T, I: IntoIterator<Item=T>>((first, rest): (T, Option<I>)) -> Vec<T> {
    merge_list_opt(first, rest)
}