//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, RangeFrom, RangeTo};
use nom::{Compare, CompareResult, InputIter, InputLength, InputTake, InputTakeAtPosition, IResult, Needed, Offset, Slice};
use nom::error::{Error, ErrorKind, ParseError};
use crate::registry::VotingRegistry;

/// The input of the parser. Allows to carry an voting registry for parsing reasons.
#[derive(Clone, Copy, Debug)]
pub struct ParserInput<'a,'b> {
    input: &'a str,
    registry: Option<&'b VotingRegistry>,
}

impl<'a> Deref for ParserInput<'a, '_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.input
    }
}

impl Display for ParserInput<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.input, f)
    }
}

impl AsRef<str> for ParserInput<'_, '_> {
    fn as_ref(&self) -> &str {
        self.input
    }
}

impl PartialEq for ParserInput<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        other.input.eq(other.input)
    }
}

impl<'a, 'b> ParserInput<'a,'b> {
    pub fn new(input: &'a str, registry: &'b VotingRegistry) -> Self {
        Self {
            input,
            registry: Some(registry)
        }
    }

    pub fn registry(&self) -> Option<&'b VotingRegistry> {
        self.registry
    }

    #[inline(always)]
    fn new_from(&self, input: &'a str) -> ParserInput<'a, 'b> {
        Self {
            registry: self.registry,
            input
        }
    }

    #[inline(always)]
    fn new_from_pair(&self, input: (&'a str, &'a str)) -> (ParserInput<'a, 'b>, ParserInput<'a, 'b>) {
        (
            Self {
                registry: self.registry,
                input: input.0
            },
            Self {
                registry: self.registry,
                input: input.1
            }
        )
    }
}

impl<'a> ParserInput<'a, 'a> {
    pub fn new_without_registry(input: &'a str) -> Self {
        Self {
            input,
            registry: None
        }
    }
}

impl InputLength for ParserInput<'_, '_> {
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}


impl<'a, 'b> InputTake for ParserInput<'a,'b> {
    fn take(&self, count: usize) -> Self {
        self.new_from(self.input.take(count))
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        self.new_from_pair(self.input.take_split(count))
    }
}

impl<'a,'b> InputTakeAtPosition for ParserInput<'a, 'b> {
    type Item = <&'a str as InputTakeAtPosition>::Item;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E> where P: Fn(Self::Item) -> bool {
        match self.input.split_at_position::<_, Error<&str>>(predicate) {
            Ok(value) => Ok(self.new_from_pair(value)),
            Err(err) => Err(
                err.map(|value| E::from_error_kind(self.new_from(value.input), value.code))
            )
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(&self, predicate: P, e: ErrorKind) -> IResult<Self, Self, E> where P: Fn(Self::Item) -> bool {
        match self.input.split_at_position1::<_, Error<&str>>(predicate, e) {
            Ok(value) => Ok(self.new_from_pair(value)),
            Err(err) => Err(
                err.map(|value| E::from_error_kind(self.new_from(value.input), value.code))
            )
        }
    }

    fn split_at_position_complete<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E> where P: Fn(Self::Item) -> bool {
        match self.input.split_at_position_complete::<_, Error<&str>>(predicate) {
            Ok(value) => Ok(self.new_from_pair(value)),
            Err(err) => Err(
                err.map(|value| E::from_error_kind(self.new_from(value.input), value.code))
            )
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(&self, predicate: P, e: ErrorKind) -> IResult<Self, Self, E> where P: Fn(Self::Item) -> bool {
        match self.input.split_at_position1_complete::<_, Error<&str>>(predicate, e) {
            Ok(value) => Ok(self.new_from_pair(value)),
            Err(err) => Err(
                err.map(|value| E::from_error_kind(self.new_from(value.input), value.code))
            )
        }
    }
}

impl<'a> InputIter for ParserInput<'a, '_> {
    type Item = <&'a str as InputIter>::Item;
    type Iter = <&'a str as InputIter>::Iter;
    type IterElem = <&'a str as InputIter>::IterElem;

    fn iter_indices(&self) -> Self::Iter {
        self.input.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.input.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize> where P: Fn(Self::Item) -> bool {
        self.input.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        self.input.slice_index(count)
    }
}

impl Slice<RangeFrom<usize>> for ParserInput<'_, '_> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.new_from(self.input.slice(range))
    }
}

impl Slice<RangeTo<usize>> for ParserInput<'_, '_> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.new_from(self.input.slice(range))
    }
}

impl Offset for ParserInput<'_, '_> {
    fn offset(&self, second: &Self) -> usize {
        self.input.offset(second.input)
    }
}

impl<'c> Compare<&'c str> for ParserInput<'_, '_> {
    fn compare(&self, t: &'c str) -> CompareResult {
        self.input.compare(t)
    }

    fn compare_no_case(&self, t: &'c str) -> CompareResult {
        self.input.compare_no_case(t)
    }
}

impl<'a> From<&'a str> for ParserInput<'a, 'a> {
    #[inline(always)]
    fn from(value: &'a str) -> Self {
        Self::new_without_registry(value)
    }
}
