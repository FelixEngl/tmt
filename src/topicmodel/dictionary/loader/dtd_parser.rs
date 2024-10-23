// pub use documents::*;
// pub use errors::*;
// pub use logical_structures::*;
// pub use physical_structures::*;
//
// mod errors {
//     use crate::topicmodel::dictionary::loader::dtd_parser::AlreadyInUseError;
//     use nom::error::ParseError;
//
//     pub trait XMLParseError<T>:
//     ParseError<T>
//     + nom::error::FromExternalError<T, AlreadyInUseError>
//     + nom::error::FromExternalError<T, strum::ParseError>
//     + nom::error::FromExternalError<T, std::char::CharTryFromError>
//     + nom::error::FromExternalError<T, std::num::ParseIntError>
//     {}
//
//     impl<S, T> XMLParseError<T> for S
//     where S: ParseError<T>
//     + nom::error::FromExternalError<T, AlreadyInUseError>
//     + nom::error::FromExternalError<T, strum::ParseError>
//     + nom::error::FromExternalError<T, std::char::CharTryFromError>
//     + nom::error::FromExternalError<T, std::num::ParseIntError>
//     {
//
//     }
// }
//
//
// mod documents {
//     pub use cardinality::*;
//     pub use cdata_sections::*;
//     pub use character_data_and_markup::*;
//     pub use characters::*;
//     pub use comments::*;
//     pub use common_syntactic_constructs::*;
//     pub use end_of_line_handling::*;
//     pub use language_identification::*;
//     pub use processing_instruction::*;
//     pub use prolog_and_xml::*;
//     pub use standalone_document_declaration::*;
//     pub use well_formed_xml_documents::*;
//     pub use white_space_handling::*;
//
//     mod well_formed_xml_documents {
//         use super::super::{element, misc, prolog, Element, Misc, Prolog, XMLParseError};
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use nom::combinator::{map, opt};
//         use nom::multi::many1;
//         use nom::sequence::tuple;
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use itertools::Itertools;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, PartialEq, Eq, Hash)]
//         pub struct Document<I>(pub Prolog<I>, pub Element<I>, pub Option<Vec<Misc<I>>>);
//
//         impl<I> ToOwned2 for Document<I> where I: ToOwned {
//             type Owned2 = Document<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Document(
//                     self.0.to_owned2(),
//                     self.1.to_owned2(),
//                     self.2.as_ref().map(
//                         |value|
//                             value.iter().map(|value| {
//                                 value.to_owned2()
//                             }).collect_vec()
//                     )
//                 )
//             }
//         }
//
//         impl<I> Display for Document<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "{}{}", self.0, self.1)?;
//                 if let Some(ref misc) = self.2 {
//                     for value in misc.iter() {
//                         write!(f, "{value}")?;
//                     }
//                 }
//                 Ok(())
//             }
//         }
//
//         pub fn document<I: DtdParserInput, E: XMLParseError<I>>(s: I) -> IResult<I, Document<I>, E> {
//             map(
//                 tuple((
//                     prolog,
//                     element,
//                     opt(many1(misc))
//                 )),
//                 |(a, b, c)| Document(a, b, c)
//             )(s)
//         }
//
//     }
//
//     mod characters {
//         pub fn is_char(c: char) -> bool {
//             matches!(
//             c,
//             '\u{9}'
//             | '\u{A}'
//             | '\u{D}'
//             | '\u{20}'..='\u{D7FF}'
//             | '\u{E000}'..='\u{FFFD}'
//             | '\u{10000}'..='\u{10FFFF}'
//         )
//         }
//     }
//
//     // todo: https://www.w3.org/TR/REC-xml/#sec-line-ends
//
//     mod common_syntactic_constructs {
//         use super::super::XMLParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         pub use literals::*;
//         pub use names_and_tokens::*;
//         use nom::character::complete::multispace0;
//         use nom::error::ParseError;
//         use nom::sequence::delimited;
//         use nom::IResult;
//
//         mod names_and_tokens {
//             use std::borrow::Borrow;
//             use super::super::super::{is_char, XMLParseError as ParseError};
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::{DtdParserInput, Merge, SupportsMerged};
//             use derive_more::From;
//             use itertools::Itertools;
//             use nom::bytes::complete::{take_while, take_while1};
//             use nom::character::complete::char;
//             use nom::combinator::{into, recognize};
//             use nom::error::ErrorKind;
//             use nom::multi::separated_list1;
//             use nom::sequence::pair;
//             use nom::{AsChar, IResult, InputTakeAtPosition, Parser};
//             use std::fmt::{Display, Formatter};
//             use std::ops::Deref;
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{GetBorrowed, ToOwned2};
//
//             pub fn dtd_char<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
//             where
//                 T: InputTakeAtPosition,
//                 <T as InputTakeAtPosition>::Item: AsChar
//             {
//                 input.split_at_position_complete(|value| is_char(value.as_char()))
//             }
//
//             pub fn dtd_char1<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
//             where
//                 T: InputTakeAtPosition,
//                 <T as InputTakeAtPosition>::Item: AsChar
//             {
//                 input.split_at_position1_complete(|value| is_char(value.as_char()), ErrorKind::Char)
//             }
//
//             pub fn is_name_start(c: char) -> bool {
//                 matches!(
//                     c,
//                     'a'..='z'
//                     |'A'..='Z'
//                     | '_'
//                     | ':'
//                     | '\u{C0}'..='\u{D6}'
//                     | '\u{D8}'..='\u{F6}'
//                     | '\u{F8}'..='\u{2FF}'
//                     | '\u{370}'..='\u{37D}'
//                     | '\u{37F}'..='\u{1FFF}'
//                     | '\u{200C}'..='\u{200D}'
//                     | '\u{2070}'..='\u{218F}'
//                     | '\u{2C00}'..='\u{2FEF}'
//                     | '\u{3001}'..='\u{D7FF}'
//                     | '\u{F900}'..='\u{FDCF}'
//                     | '\u{FDF0}'..='\u{FFFD}'
//                     | '\u{10000}'..='\u{EFFFF}'
//                 )
//             }
//
//             pub fn is_name_char(c: char) -> bool {
//                 is_name_start(c)
//                     || matches!(c, '-' | '.' | '0'..='9' | '\u{B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}')
//             }
//
//
//             #[derive(From, Clone, Debug, Eq, PartialEq, Hash)]
//             #[repr(transparent)]
//             pub struct Name<I>(pub I);
//
//             impl<I> SupportsMerged for Name<I>
//             where
//                 I: ToOwned,
//                 <I as ToOwned>::Owned: Merge<<I as ToOwned>::Owned, Merged=<I as ToOwned>::Owned>
//             {
//                 type Merged = I::Owned;
//
//                 fn get_merged(&self) -> Self::Merged {
//                     self.0.to_owned()
//                 }
//             }
//
//
//             impl<I> ToOwned2 for Name<I> where I: ToOwned {
//                 type Owned2 = Name<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     Name(self.0.to_owned())
//                 }
//             }
//
//             pub fn name<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, Name<I>, E>
//             {
//                 into(
//                     recognize(
//                         pair::<I, I, I, E, _, _>(
//                             take_while1(is_name_start),
//                             take_while(is_name_char)
//                         )
//                     )
//                 )(s)
//             }
//
//             impl<I> Borrow<I> for Name<I> {
//                 fn borrow(&self) -> &I {
//                     &self.0
//                 }
//             }
//
//             impl<I> Deref for Name<I> where I: AsRef<str> {
//                 type Target = str;
//
//                 fn deref(&self) -> &Self::Target {
//                     self.0.as_ref()
//                 }
//             }
//
//             impl<I> Display for Name<I> where I: Display {
//                 delegate::delegate! {
//                     to self.0 {
//                         fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                     }
//                 }
//             }
//
//             #[derive(Clone, Debug, From)]
//             #[repr(transparent)]
//             pub struct Names<I>(pub Vec<Name<I>>);
//
//             impl<I> ToOwned2 for Names<I> where I: ToOwned {
//                 type Owned2 = Names<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     Names(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//                 }
//             }
//
//             pub fn names<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, Names<I>, E>
//
//             {
//                 into(
//                     separated_list1(
//                         char::<I, E>('\u{20}'),
//                         name
//                     )
//                 )(s)
//             }
//
//             impl<I> Deref for Names<I> {
//                 type Target = [Name<I>];
//
//                 fn deref(&self) -> &Self::Target {
//                     &self.0
//                 }
//             }
//
//             impl<I> Display for Names<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "{}", self.0.iter().join("\u{20}"))
//                 }
//             }
//
//             #[derive(Clone, Hash, Eq, PartialEq, Debug, From)]
//             #[repr(transparent)]
//             pub struct Nmtoken<I>(pub I);
//
//             impl<I> ToOwned2 for Nmtoken<I> where I: ToOwned {
//                 type Owned2 = Nmtoken<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     Nmtoken(self.0.to_owned())
//                 }
//             }
//
//             pub fn nm_token<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, Nmtoken<I>, E> {
//                 into(take_while1::<_, I, E>(is_name_char))(s)
//             }
//
//             impl<I> Borrow<I> for Nmtoken<I> {
//                 fn borrow(&self) -> &I {
//                     &self.0
//                 }
//             }
//
//             impl<I> Deref for Nmtoken<I> where I: AsRef<str> {
//                 type Target = str;
//
//                 fn deref(&self) -> &Self::Target {
//                     self.0.as_ref()
//                 }
//             }
//
//             impl<I> Display for Nmtoken<I> where I: Display {
//                 delegate::delegate! {
//                     to self.0 {
//                         fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                     }
//                 }
//             }
//
//             #[derive(Clone, Debug, From)]
//             #[repr(transparent)]
//             pub struct Nmtokens<I>(Vec<Nmtoken<I>>);
//
//             pub fn nm_tokens<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, Nmtokens<I>, E> {
//                 into(
//                     separated_list1(
//                         char::<I, E>('\u{20}'),
//                         nm_token::<I, E>
//                     ),
//                 )(s)
//             }
//
//             impl<I> ToOwned2 for Nmtokens<I> where I: ToOwned {
//                 type Owned2 = Nmtokens<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     Nmtokens(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//                 }
//             }
//
//             impl<I> Deref for Nmtokens<I> {
//                 type Target = [Nmtoken<I>];
//
//                 fn deref(&self) -> &Self::Target {
//                     &self.0
//                 }
//             }
//
//             impl<I> Display for Nmtokens<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "{}", self.0.iter().join("\u{20}"))
//                 }
//             }
//
//         }
//
//         mod literals {
//             use super::super::super::physical_structures::{pe_reference, reference, PEReference, Reference};
//             use super::super::super::XMLParseError as ParseError;
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use derive_more::From;
//             use itertools::Itertools;
//             use nom::branch::alt;
//             use nom::bytes::complete::{take_while, take_while1};
//             use nom::character::complete::char;
//             use nom::combinator::{into, recognize};
//             use nom::multi::many0;
//             use nom::sequence::delimited;
//             use nom::{IResult, Parser};
//             use std::fmt::{Display, Formatter};
//             use std::ops::Deref;
//             use strum::Display;
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             fn is_pub_id_char(c: char) -> bool {
//                 matches!(
//                     c,
//                     '\u{20}'
//                     | '\u{D}'
//                     | '\u{A}'
//                     | 'a'..='z'
//                     | 'A'..='Z'
//                     | '0'..='9'
//                 ) || "-'()+,./:=?;!*#@$_%".contains(c)
//             }
//
//             fn is_pub_id_char_no_apostroph(c: char) -> bool {
//                 matches!(
//                     c,
//                     '\u{20}'
//                     | '\u{D}'
//                     | '\u{A}'
//                     | 'a'..='z'
//                     | 'A'..='Z'
//                     | '0'..='9'
//                 ) || "-()+,./:=?;!*#@$_%".contains(c)
//             }
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//             pub enum EntityValuePart<I> {
//                 #[strum(to_string = "{0}")]
//                 Raw(I),
//                 #[strum(to_string = "{0}")]
//                 PEReference(PEReference<I>),
//                 #[strum(to_string = "{0}")]
//                 Reference(Reference<I>),
//             }
//
//
//             impl<I> ToOwned2 for EntityValuePart<I> where I: ToOwned {
//                 type Owned2 = EntityValuePart<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         EntityValuePart::Raw(value) => {
//                             EntityValuePart::Raw(value.to_owned())
//                         }
//                         EntityValuePart::PEReference(value) => {
//                             EntityValuePart::PEReference(value.to_owned2())
//                         }
//                         EntityValuePart::Reference(value) => {
//                             EntityValuePart::Reference(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//             fn is_raw_entity_value_part(delimiter: char) -> impl Fn(char) -> bool {
//                 move |c| {
//                     c != delimiter && c != '%' && c != '&'
//                 }
//             }
//
//             pub fn entity_value_part<I: DtdParserInput, E: ParseError<I>>(delimiter: char) -> impl Parser<I, EntityValuePart<I>, E>
//
//             {
//                 alt((
//                     into(recognize(take_while1::<_, I, E>(is_raw_entity_value_part(delimiter)))),
//                     into(pe_reference::<_, E>),
//                     into(reference::<_, E>),
//                 ))
//             }
//
//             macro_rules! value_display {
//                 ($name: ty) => {
//                     impl<I> Display for $name where I: Display + AsRef<str> {
//                         fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                             if self.contains_str("\"") {
//                                 write!(f, "'{}'", self.0.iter().join(""))
//                             } else {
//                                 write!(f, "\"{}\"", self.0.iter().join(""))
//                             }
//                         }
//                     }
//                 };
//             }
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//             #[repr(transparent)]
//             pub struct EntityValue<I>(pub Vec<EntityValuePart<I>>);
//
//             impl<I> EntityValue<I> {
//                 pub fn iter(&self) -> std::slice::Iter<EntityValuePart<I>> {
//                     self.0.iter()
//                 }
//             }
//
//             impl<I> ToOwned2 for EntityValue<I> where I: ToOwned {
//                 type Owned2 = EntityValue<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     EntityValue(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//                 }
//             }
//
//             impl<I> EntityValue<I> where I: AsRef<str> {
//                 pub fn contains_str(&self, s: &str) -> bool {
//                     self.0.iter().any(
//                         |value| {
//                             match value {
//                                 EntityValuePart::Raw(value) => {
//                                     value.as_ref().contains(s)
//                                 }
//                                 EntityValuePart::PEReference(value) => {
//                                     value.contains(s)
//                                 }
//                                 EntityValuePart::Reference(value) => {
//                                     match value {
//                                         Reference::EntityRef(value) => {
//                                             value.contains(s)
//                                         }
//                                         Reference::CharRef(value) => {
//                                             s.chars().exactly_one().is_ok_and(|c| c == value.as_char())
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     )
//                 }
//             }
//
//             pub fn entity_value<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EntityValue<I>, E> {
//                 into(
//                     alt((
//                         delimited(
//                             char::<I, E>('"'),
//                             many0(entity_value_part('"')),
//                             char::<I, E>('"'),
//                         ),
//                         delimited(
//                             char::<I, E>('\''),
//                             many0(entity_value_part('\'')),
//                             char::<I, E>('\''),
//                         )
//                     ))
//                 )(s)
//             }
//
//             value_display!(EntityValue<I>);
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//             pub enum AttValuePart<I> {
//                 #[strum(to_string = "{0}")]
//                 Raw(I),
//                 #[strum(to_string = "{0}")]
//                 Reference(Reference<I>)
//             }
//
//             impl<I> ToOwned2 for AttValuePart<I> where I: ToOwned {
//                 type Owned2 = AttValuePart<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         AttValuePart::Raw(value) => {
//                             AttValuePart::Raw(value.to_owned())
//                         }
//                         AttValuePart::Reference(value) => {
//                             AttValuePart::Reference(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//
//             fn is_raw_att_value_part(delimiter: char) -> impl Fn(char) -> bool {
//                 move |c| {
//                     c != delimiter && c != '<' && c != '&'
//                 }
//             }
//
//             pub fn att_value_part<I: DtdParserInput, E: ParseError<I>>(delimiter: char) -> impl Parser<I, AttValuePart<I>, E> {
//                 alt((
//                     into(recognize(take_while1::<_, I, E>(is_raw_att_value_part(delimiter)))),
//                     into(reference::<_, E>),
//                 ))
//             }
//
//             #[derive(Debug, Eq, Clone, PartialEq, Hash, From)]
//             #[repr(transparent)]
//             pub struct AttValue<I>(pub Vec<AttValuePart<I>>);
//
//             impl<I> ToOwned2 for AttValue<I> where I: ToOwned {
//                 type Owned2 = AttValue<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     AttValue(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//                 }
//             }
//
//             pub fn att_value<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, AttValue<I>, E> {
//                 into(
//                     alt((
//                         delimited(
//                             char::<I, E>('"'),
//                             many0(att_value_part('"')),
//                             char::<I, E>('"'),
//                         ),
//                         delimited(
//                             char::<I, E>('\''),
//                             many0(att_value_part('\'')),
//                             char::<I, E>('\''),
//                         )
//                     ))
//                 )(s)
//             }
//
//             impl<I> AttValue<I> where I: AsRef<str> {
//                 pub fn contains_str(&self, s: &str) -> bool {
//                     self.0.iter().any(
//                         |value| {
//                             match value {
//                                 AttValuePart::Raw(value) => {
//                                     value.as_ref().contains(s)
//                                 }
//                                 AttValuePart::Reference(value) => {
//                                     match value {
//                                         Reference::EntityRef(value) => {
//                                             value.contains(s)
//                                         }
//                                         Reference::CharRef(value) => {
//                                             s.chars().exactly_one().is_ok_and(|c| c == value.as_char())
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     )
//                 }
//             }
//
//             value_display!(AttValue<I>);
//
//
//             macro_rules! literal_display {
//                 ($name: ident) => {
//                     impl Display for $name {
//                         fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                             if self.0.contains('"') {
//                                 write!(f, "'{}'", self.0)
//                             } else {
//                                 write!(f, "\"{}\"", self.0)
//                             }
//                         }
//                     }
//                 };
//             }
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//             #[repr(transparent)]
//             pub struct SystemLiteral<I>(pub I);
//
//             impl<I> ToOwned2 for SystemLiteral<I> where I: ToOwned {
//                 type Owned2 = SystemLiteral<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     SystemLiteral(self.0.to_owned())
//                 }
//             }
//
//             impl<I> Display for SystemLiteral<I> where I: Display {
//                 delegate::delegate! {
//                     to self.0 {
//                         fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                     }
//                 }
//             }
//
//
//             impl<I> Deref for SystemLiteral<I> where I: AsRef<str> {
//                 type Target = str;
//
//                 fn deref(&self) -> &Self::Target {
//                     self.0.as_ref()
//                 }
//             }
//
//
//             pub fn system_literal<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, SystemLiteral<I>, E> {
//                 into(
//                     alt((
//                         delimited(
//                             char::<I, E>('"'),
//                             take_while(|value| value != '"'),
//                             char::<I, E>('"'),
//                         ),
//                         delimited(
//                             char::<I, E>('\''),
//                             take_while(|value| value != '\''),
//                             char::<I, E>('\''),
//                         )
//                     ))
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//             #[repr(transparent)]
//             pub struct PubidLiteral<I>(pub I);
//
//             impl<I> ToOwned2 for PubidLiteral<I> where I: ToOwned {
//                 type Owned2 = PubidLiteral<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     PubidLiteral(self.0.to_owned())
//                 }
//             }
//
//             impl<I> Display for PubidLiteral<I> where I: Display {
//                  delegate::delegate! {
//                      to self.0 {
//                          fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                      }
//                  }
//             }
//
//             impl<I> Deref for PubidLiteral<I> where I: AsRef<str> {
//                 type Target = str;
//
//                 fn deref(&self) -> &Self::Target {
//                     self.0.as_ref()
//                 }
//             }
//
//             pub fn pub_id_literal<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, PubidLiteral<I>, E> {
//                 into(
//                     alt((
//                         delimited(
//                             char::<I, E>('"'),
//                             take_while(is_pub_id_char),
//                             char::<I, E>('"'),
//                         ),
//                         delimited(
//                             char::<I, E>('\''),
//                             take_while(is_pub_id_char_no_apostroph),
//                             char::<I, E>('\''),
//                         )
//                     ))
//                 )(s)
//             }
//         }
//
//         // customs
//         pub fn eq<I: DtdParserInput, E: XMLParseError<I>>(s: I) -> IResult<I, char, E> {
//             delimited(
//                 multispace0,
//                 nom::character::complete::char('='),
//                 multispace0
//             )(s)
//         }
//     }
//
//     mod character_data_and_markup {
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use derive_more::From;
//         use nom::bytes::complete::take_while;
//         use nom::combinator::into;
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//         #[repr(transparent)]
//         pub struct CharData<I>(pub I);
//
//         impl<I> ToOwned2 for CharData<I> where I: ToOwned {
//             type Owned2 = CharData<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 CharData(self.0.to_owned())
//             }
//         }
//
//         impl<I> Display for CharData<I> where I: Display {
//             delegate::delegate! {
//                 to self.0 {
//                     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                 }
//             }
//         }
//
//         pub fn char_data<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, CharData<I>, E> {
//             into(
//                 nom::combinator::verify(
//                     take_while::<_, I, E>(|value: char| value != '<' && value != '&'),
//                     |value: &I| !value.contains("]]>")
//                 )
//             )(s)
//         }
//     }
//
//     mod comments {
//         use super::super::XMLParseError as ParseError;
//         use super::characters::is_char;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use derive_more::From;
//         use nom::branch::alt;
//         use nom::bytes::complete::{tag, take_while1};
//         use nom::character::complete::char;
//         use nom::combinator::{into, recognize};
//         use nom::multi::many0;
//         use nom::sequence::{delimited, pair};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//         #[repr(transparent)]
//         pub struct Comment<I>(pub I);
//
//         impl<I> ToOwned2 for Comment<I> where I: ToOwned {
//             type Owned2 = Comment<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Comment(self.0.to_owned())
//             }
//         }
//
//
//         impl<I> Display for Comment<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<!--{}-->", self.0)
//             }
//         }
//
//         fn is_comment_char(c: char) -> bool {
//             c != '-' && is_char(c)
//         }
//
//         pub fn comment<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Comment<I>, E> {
//             into(
//                 delimited(
//                     tag::<_, I, E>("<!--"),
//                     recognize(many0(
//                         alt((
//                             take_while1(is_comment_char),
//                             recognize(pair(char('-'), take_while1(is_comment_char)))
//                         ))
//                     )),
//                     tag("-->"),
//                 )
//             )(s)
//         }
//     }
//
//     mod processing_instruction {
//         use super::super::XMLParseError as ParseError;
//         use super::{name, Name};
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use derive_more::From;
//         use nom::bytes::complete::tag;
//         use nom::bytes::complete::take_until1;
//         use nom::character::complete::multispace1;
//         use nom::combinator::{into, map, opt, verify};
//         use nom::sequence::{delimited, pair, preceded};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//         #[repr(transparent)]
//         pub struct PITarget<I>(pub Name<I>);
//
//         impl<I> ToOwned2 for PITarget<I> where I: ToOwned {
//             type Owned2 = PITarget<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 PITarget(self.0.to_owned2())
//             }
//         }
//
//         impl<I> Display for PITarget<I> where  I: Display {
//             delegate::delegate! {
//                 to self.0 {
//                     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                 }
//             }
//         }
//
//         pub fn pi_target<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, PITarget<I>, E> {
//             into(
//                 verify(
//                     name::<I, E>,
//                     |value: &I| !value.as_ref().eq_ignore_ascii_case("xml")
//                 )
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//         pub struct PI<I>(
//             pub PITarget<I>,
//             pub Option<I>
//         );
//
//         impl<I> ToOwned2 for PI<I> where I: ToOwned {
//             type Owned2 = PI<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 PI(self.0.to_owned2(), self.1.as_ref().map(ToOwned::to_owned))
//             }
//         }
//
//         impl<I> Display for PI<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<?{}", self.0)?;
//                 if let Some(ref s) = self.1 {
//                     write!(f, " {s}")?;
//                 }
//                 write!(f, "?>")
//             }
//         }
//
//         pub fn pi<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, PI<I>, E> {
//             map(
//                 delimited(
//                     tag::<_, I, E>("<?"),
//                     pair(
//                         pi_target::<I, E>,
//                         opt(
//                             preceded(
//                                 multispace1,
//                                 take_until1("?>")
//                             )
//                         )
//                     ),
//                     tag::<_, I, E>("?>"),
//                 ),
//                 |(a, b): (PITarget<I>, Option<I>)| PI(a, b)
//             )(s)
//         }
//     }
//
//     mod cdata_sections {
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use derive_more::From;
//         use nom::bytes::complete::{tag, take_until1};
//         use nom::combinator::into;
//         use nom::sequence::delimited;
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//         #[repr(transparent)]
//         pub struct CDSect<I>(pub I);
//
//         impl<I> ToOwned2 for CDSect<I> where I: ToOwned {
//             type Owned2 = CDSect<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 CDSect(self.0.to_owned())
//             }
//         }
//
//         impl<I> Display for CDSect<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<!CDATA[{}]]>", self.0)
//             }
//         }
//
//         pub fn cd_sect<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, CDSect<I>, E> {
//             into(
//                 delimited(
//                     tag::<_, I, E>("<!CDATA["),
//                     take_until1("]]>"),
//                     tag::<_, I, E>("]]>")
//                 )
//             )(s)
//         }
//     }
//
//     mod prolog_and_xml {
//         pub use document_type_definition::*;
//         pub use external_subset::*;
//         pub use prolog::*;
//
//         mod prolog {
//             use super::super::super::physical_structures::{encoding_decl, EncodingDecl};
//             use super::super::super::XMLParseError as ParseError;
//             use super::super::comments::{comment, Comment};
//             use super::super::common_syntactic_constructs::eq;
//             use super::super::processing_instruction::{pi, PI};
//             use super::super::standalone_document_declaration::{sd_decl, SDDecl};
//             use super::{doc_type_decl, DocTypeDecl};
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use derive_more::From;
//             use itertools::Itertools;
//             use nom::branch::alt;
//             use nom::bytes::complete::tag;
//             use nom::character::complete::{char, multispace1};
//             use nom::combinator::{into, map, map_res, opt, recognize, value};
//             use nom::multi::many0;
//             use nom::sequence::{delimited, pair, preceded, tuple};
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use thiserror::Error;
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub struct Prolog<I>(
//                 pub Option<XMLDecl<I>>,
//                 pub Vec<Misc<I>>,
//                 pub Option<(DocTypeDecl<I>, Vec<Misc<I>>)>
//             );
//
//             impl<I> ToOwned2 for Prolog<I> where I: ToOwned {
//                 type Owned2 = Prolog<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     Prolog(
//                         self.0.as_ref().map(ToOwned2::to_owned2),
//                         self.1.iter().map(ToOwned2::to_owned2).collect_vec(),
//                         self.2.as_ref().map(|(a, b)| (a.to_owned2(), b.iter().map(ToOwned2::to_owned2).collect_vec()))
//                     )
//                 }
//             }
//
//             impl<I> Display for Prolog<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     if let Some(ref xml_decl) = self.0 {
//                         write!(f, "{xml_decl}")?;
//                     }
//                     write!(f, "{}", self.1.iter().join(""))?;
//                     if let Some(ref doc_type_decl) = self.2 {
//                         write!(f, "{}{}", doc_type_decl.0, doc_type_decl.1.iter().join(""))?;
//                     }
//                     Ok(())
//                 }
//             }
//
//             pub fn prolog<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Prolog<I>, E> {
//                 map(
//                     tuple((
//                         opt(xml_decl),
//                         many0(misc),
//                         opt(pair(doc_type_decl, many0(misc)))
//                     )),
//                     |(a, b, c)| Prolog(a, b, c)
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub struct XMLDecl<I>(
//                 pub VersionInfo<I>,
//                 pub Option<EncodingDecl<I>>,
//                 pub Option<SDDecl>
//             );
//
//             impl<I> ToOwned2 for XMLDecl<I> where I: ToOwned {
//                 type Owned2 = XMLDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     XMLDecl(
//                         self.0.to_owned2(),
//                         self.1.as_ref().map(ToOwned2::to_owned2),
//                         self.2.clone()
//                     )
//                 }
//             }
//
//             impl<I> Display for XMLDecl<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "<?xml{}", self.0)?;
//                     if let Some(enc) = &self.1 {
//                         write!(f, "{enc}")?;
//                     }
//                     if let Some(enc) = self.2 {
//                         write!(f, "{enc}")?;
//                     }
//                     write!(f, "?>")
//                 }
//             }
//
//             #[derive(Debug, Error)]
//             #[error("The {0} was declared multiple times!")]
//             pub struct AlreadyInUseError(&'static str);
//
//             pub fn xml_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, XMLDecl<I>, E> {
//                 delimited(
//                     tag("<?xml"),
//                     map_res(
//                         tuple(
//                             (
//                                 version_info,
//                                 opt(enc_or_sd),
//                                 opt(enc_or_sd)
//                             )
//                         ),
//                         |(version_info, b, c)| {
//                             match (b, c) {
//                                 (None, None) => Ok(XMLDecl(version_info, None, None)),
//                                 (Some(EncDeclOrSDDecl::SD(c)), None) | (None, Some(EncDeclOrSDDecl::SD(c))) => Ok(XMLDecl(version_info, None, Some(c))),
//                                 (None, Some(EncDeclOrSDDecl::Enc(b))) | (Some(EncDeclOrSDDecl::Enc(b)), None) => Ok(XMLDecl(version_info, Some(b), None)),
//                                 (Some(EncDeclOrSDDecl::SD(c)), Some(EncDeclOrSDDecl::Enc(b))) | (Some(EncDeclOrSDDecl::Enc(b)), Some(EncDeclOrSDDecl::SD(c))) => Ok(XMLDecl(version_info, Some(b), Some(c))),
//                                 (Some(EncDeclOrSDDecl::Enc(_)), Some(EncDeclOrSDDecl::Enc(_))) => Err(AlreadyInUseError("The encoding was declared multiple times!")),
//                                 (Some(EncDeclOrSDDecl::SD(_)), Some(EncDeclOrSDDecl::SD(_))) => Err(AlreadyInUseError("The standalone was declared multiple times!")),
//                             }
//                         }
//                     ),
//                     tag("?>")
//                 )(s)
//             }
//
//             enum EncDeclOrSDDecl<I> {
//                 Enc(EncodingDecl<I>),
//                 SD(SDDecl)
//             }
//
//             impl<I> ToOwned2 for EncDeclOrSDDecl<I> where I: ToOwned {
//                 type Owned2 = EncDeclOrSDDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         EncDeclOrSDDecl::Enc(value) => {
//                             EncDeclOrSDDecl::Enc(value.to_owned2())
//                         }
//                         EncDeclOrSDDecl::SD(value) => {
//                             EncDeclOrSDDecl::SD(value.clone())
//                         }
//                     }
//                 }
//             }
//
//             fn enc_or_sd<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EncDeclOrSDDecl<I>, E> {
//                 alt((
//                     map(encoding_decl, EncDeclOrSDDecl::Enc),
//                     map(sd_decl, EncDeclOrSDDecl::SD),
//                 ))(s)
//             }
//
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             #[repr(transparent)]
//             pub struct VersionInfo<I>(pub VersionNum<I>);
//
//             impl<I> ToOwned2 for VersionInfo<I> where I: ToOwned {
//                 type Owned2 = VersionInfo<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     VersionInfo(self.0.to_owned2())
//                 }
//             }
//
//             impl<I> Display for VersionInfo<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, " version=\"{}\"", self.0)
//                 }
//             }
//
//             pub fn version_info<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, VersionInfo<I>, E> {
//                 map(
//                     preceded(
//                         delimited(multispace1, tag("version"), eq),
//                         alt((
//                             delimited(
//                                 char('"'),
//                                 version_num,
//                                 char('"'),
//                             ),
//                             delimited(
//                                 char('\''),
//                                 version_num,
//                                 char('\''),
//                             )
//                         ))
//                     ),
//                     VersionInfo
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//             #[repr(transparent)]
//             pub struct VersionNum<I>(pub I);
//
//             impl<I> ToOwned2 for VersionNum<I> where I: ToOwned {
//                 type Owned2 = VersionNum<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     VersionNum(self.0.to_owned())
//                 }
//             }
//
//
//             impl<I> Display for VersionNum<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "{}", self.0)
//                 }
//             }
//
//             pub fn version_num<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, VersionNum<I>, E> {
//                 into(
//                     recognize(
//                         preceded(
//                             tag("1."),
//                             nom::character::complete::digit1::<I, E>
//                         )
//                     ),
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub enum Misc<I> {
//                 Comment(Comment<I>),
//                 PI(PI<I>),
//                 Space
//             }
//
//             impl<I> ToOwned2 for Misc<I> where I: ToOwned {
//                 type Owned2 = Misc<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         Misc::Comment(value) => {
//                             Misc::Comment(value.to_owned2())
//                         }
//                         Misc::PI(value) => {
//                             Misc::PI(value.to_owned2())
//                         }
//                         Misc::Space => {
//                             Misc::Space
//                         }
//                     }
//                 }
//             }
//
//             impl<I> Display for Misc<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     match self {
//                         Misc::Comment(value) => {
//                             write!(f, "{value}")
//                         }
//                         Misc::PI(value) => {
//                             write!(f, "{value}")
//                         }
//                         Misc::Space => {
//                             write!(f, " ")
//                         }
//                     }
//                 }
//             }
//
//             pub fn misc<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Misc<I>, E> {
//                 alt((
//                     map(comment, Misc::Comment),
//                     map(pi, Misc::PI),
//                     value(Misc::Space, multispace1)
//                 ))(s)
//             }
//         }
//
//         mod document_type_definition {
//             use super::super::super::logical_structures::{attlist_decl, element_decl, AttlistDecl, ElementDecl};
//             use super::super::super::physical_structures::{
//                 entity_decl,
//                 external_id,
//                 notation_decl,
//                 pe_reference,
//                 EntityDecl,
//                 ExternalID,
//                 NotationDecl,
//                 PEReference
//             };
//             use super::super::super::XMLParseError as ParseError;
//             use super::super::{comment, Comment};
//             use super::super::{name, Name};
//             use super::super::{pi, PI};
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use derive_more::From;
//             use itertools::Itertools;
//             use nom::branch::alt;
//             use nom::bytes::complete::tag;
//             use nom::character::complete::{char, multispace0, multispace1};
//             use nom::combinator::{into, map, opt, value};
//             use nom::multi::many0;
//             use nom::sequence::{delimited, preceded, terminated, tuple};
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use std::hash::Hash;
//             use std::iter::FlatMap;
//             use strum::Display;
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, Display)]
//             pub enum DeclSep<I> {
//                 #[strum(to_string=" ")]
//                 Space,
//                 #[strum(to_string="{0}")]
//                 PEReference(PEReference<I>)
//             }
//
//             impl<I> ToOwned2 for DeclSep<I> where I: ToOwned {
//                 type Owned2 = DeclSep<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         DeclSep::Space => {
//                             DeclSep::Space
//                         }
//                         DeclSep::PEReference(value) => {
//                             DeclSep::PEReference(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//
//             pub fn decl_sep<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, DeclSep<I>, E> {
//                 alt((
//                     map(pe_reference, DeclSep::PEReference),
//                     value(DeclSep::Space, multispace1)
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
//             pub enum MarkUpDecl<I> {
//                 ElementDecl(ElementDecl<I>),
//                 AttlistDecl(AttlistDecl<I>),
//                 EntityDecl(EntityDecl<I>),
//                 NotationDecl(NotationDecl<I>),
//                 PI(PI<I>),
//                 Comment(Comment<I>),
//             }
//
//             impl<I> ToOwned2 for MarkUpDecl<I> where I: ToOwned {
//                 type Owned2 = MarkUpDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         MarkUpDecl::ElementDecl(value) => {
//                             MarkUpDecl::ElementDecl(value.to_owned2())
//                         }
//                         MarkUpDecl::AttlistDecl(value) => {
//                             MarkUpDecl::AttlistDecl(value.to_owned2())
//                         }
//                         MarkUpDecl::EntityDecl(value) => {
//                             MarkUpDecl::EntityDecl(value.to_owned2())
//                         }
//                         MarkUpDecl::NotationDecl(value) => {
//                             MarkUpDecl::NotationDecl(value.to_owned2())
//                         }
//                         MarkUpDecl::PI(value) => {
//                             MarkUpDecl::PI(value.to_owned2())
//                         }
//                         MarkUpDecl::Comment(value) => {
//                             MarkUpDecl::Comment(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//             pub fn mark_up_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, MarkUpDecl<I>, E> {
//                 alt((
//                     into(element_decl::<_, E>),
//                     into(attlist_decl::<_, E>),
//                     into(entity_decl::<_, E>),
//                     into(notation_decl::<_, E>),
//                     into(pi::<_, E>),
//                     into(comment::<_, E>),
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
//             pub enum IntSubsetPart<I> {
//                 #[strum(to_string="{0}")]
//                 MarkupDecl(MarkUpDecl<I>),
//                 #[strum(to_string="{0}")]
//                 DeclSep(DeclSep<I>)
//             }
//
//             impl<I> ToOwned2 for IntSubsetPart<I> where I: ToOwned {
//                 type Owned2 = IntSubsetPart<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         IntSubsetPart::MarkupDecl(value) => {
//                             IntSubsetPart::MarkupDecl(value.to_owned2())
//                         }
//                         IntSubsetPart::DeclSep(value) => {
//                             IntSubsetPart::DeclSep(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//             fn int_subset_part<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, IntSubsetPart<I>, E> {
//                 alt((
//                     into(mark_up_decl::<_, E>),
//                     into(decl_sep::<_, E>),
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//             #[repr(transparent)]
//             pub struct IntSubset<I>(pub Vec<IntSubsetPart<I>>);
//
//             impl<I> ToOwned2 for IntSubset<I> where I: ToOwned {
//                 type Owned2 = IntSubset<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     IntSubset(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//                 }
//             }
//
//             impl<I> IntSubset<I> {
//                 pub fn iter(&self) -> std::slice::Iter<IntSubsetPart<I>> {
//                     self.0.iter()
//                 }
//             }
//
//             impl<I> Display for IntSubset<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "{}", self.0.iter().join(""))
//                 }
//             }
//
//             pub fn int_subset<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, IntSubset<I>, E> {
//                 into(many0(int_subset_part::<I, E>))(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub struct DocTypeDecl<I>(
//                 pub Name<I>,
//                 pub Option<ExternalID<I>>,
//                 pub Option<IntSubset<I>>
//             );
//
//             impl<I> ToOwned2 for DocTypeDecl<I> where I: ToOwned {
//                 type Owned2 = DocTypeDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     DocTypeDecl(
//                         self.0.to_owned2(),
//                         self.1.as_ref().map(ToOwned2::to_owned2),
//                         self.2.as_ref().map(ToOwned2::to_owned2),
//                     )
//                 }
//             }
//
//             impl<I> DocTypeDecl<I> {
//                 pub fn iter<'a>(&'a self) -> FlatMap<std::option::Iter<'a, IntSubset<I>>, std::slice::Iter<'a, IntSubsetPart<I>>, impl FnMut(&'a IntSubset<I>) -> std::slice::Iter<'a, IntSubsetPart<I>>> {
//                     self.2.iter().flat_map(|value| value.iter())
//                 }
//             }
//
//             impl<I> Display for DocTypeDecl<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "<!DOCTYPE {}", self.0)?;
//                     if let Some(ref ext) = self.1 {
//                         write!(f, " {}", ext)?;
//                     }
//                     if let Some(ref sub) = self.2 {
//                         write!(f, " [{}]", sub)?;
//                     }
//                     write!(f, ">")
//                 }
//             }
//
//             pub fn doc_type_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, DocTypeDecl<I>, E> {
//                 map(
//                     delimited(
//                         terminated(tag("<!DOCTYPE"), multispace1),
//                         tuple((
//                             name,
//                             opt(preceded(multispace1, external_id)),
//                             preceded(multispace0, opt(delimited(char('['), int_subset, char(']'))))
//                         )),
//                         char('>')
//                     ),
//                     |(name, external_id, int_subset)| {
//                         DocTypeDecl (name, external_id, int_subset)
//                     }
//                 )(s)
//             }
//
//             #[inline(always)]
//             pub fn doc_type_no_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, IntSubset<I>, E> {
//                 int_subset(s)
//             }
//         }
//
//         mod external_subset {
//             use super::super::super::logical_structures::{conditional_sect, ConditionalSect};
//             use super::super::super::physical_structures::{text_decl, TextDecl};
//             use super::super::super::XMLParseError as ParseError;
//             use super::super::prolog_and_xml::{decl_sep, mark_up_decl, DeclSep, MarkUpDecl};
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use derive_more::From;
//             use itertools::Itertools;
//             use nom::branch::alt;
//             use nom::combinator::{into, map, opt};
//             use nom::multi::many0;
//             use nom::sequence::pair;
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use strum::Display;
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub struct ExtSubset<I>(
//                 pub Option<TextDecl<I>>,
//                 pub ExtSubsetDecl<I>,
//             );
//
//             impl<I> ToOwned2 for ExtSubset<I> where I: ToOwned {
//                 type Owned2 = ExtSubset<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     ExtSubset(self.0.as_ref().map(ToOwned2::to_owned2), self.1.to_owned2())
//                 }
//             }
//
//             impl<I> Display for ExtSubset<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     if let Some(ref v) = self.0 {
//                         write!(f, "{v}")?;
//                     }
//                     write!(f, "{}", self.1)
//                 }
//             }
//
//             pub fn ext_subset<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ExtSubset<I>, E> {
//                 map(
//                     pair(
//                         opt(text_decl),
//                         ext_subset_decl
//                     ),
//                     |(a, b)| ExtSubset(a, b)
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
//             pub enum ExtSubsetDeclPart<I> {
//                 #[strum(to_string = "{0}")]
//                 MarkUpDecl(MarkUpDecl<I>),
//                 #[strum(to_string = "{0}")]
//                 ConditionalSect(ConditionalSect<I>),
//                 #[strum(to_string = "{0}")]
//                 DeclSep(DeclSep<I>)
//             }
//
//             impl<I> ToOwned2 for ExtSubsetDeclPart<I> where I: ToOwned {
//                 type Owned2 = ExtSubsetDeclPart<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         ExtSubsetDeclPart::MarkUpDecl(value) => {
//                             ExtSubsetDeclPart::MarkUpDecl(value.to_owned2())
//                         }
//                         ExtSubsetDeclPart::ConditionalSect(value) => {
//                             ExtSubsetDeclPart::ConditionalSect(value.to_owned2())
//                         }
//                         ExtSubsetDeclPart::DeclSep(value) => {
//                             ExtSubsetDeclPart::DeclSep(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//             pub fn ext_subset_decl_part<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ExtSubsetDeclPart<I>, E> {
//                 alt((
//                     into(mark_up_decl::<I, E>),
//                     into(conditional_sect::<I, E>),
//                     into(decl_sep::<I, E>),
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//             pub struct ExtSubsetDecl<I>(pub Vec<ExtSubsetDeclPart<I>>);
//
//             impl<I> ToOwned2 for ExtSubsetDecl<I> where I: ToOwned {
//                 type Owned2 = ExtSubsetDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     ExtSubsetDecl(
//                         self.0
//                             .iter()
//                             .map(|value| value.to_owned2())
//                             .collect_vec()
//                     )
//                 }
//             }
//
//             impl<I> Display for ExtSubsetDecl<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "{}", self.0.iter().join(""))
//                 }
//             }
//
//             pub fn ext_subset_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ExtSubsetDecl<I>, E> {
//                 into(many0(ext_subset_decl_part::<I, E>))(s)
//             }
//         }
//     }
//
//     mod standalone_document_declaration {
//         use super::super::XMLParseError as ParseError;
//         use super::common_syntactic_constructs::eq;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use nom::branch::alt;
//         use nom::bytes::complete::{is_not, tag};
//         use nom::character::complete::{char, multispace1};
//         use nom::combinator::map_res;
//         use nom::sequence::{delimited, preceded};
//         use nom::IResult;
//         use strum::{Display, EnumString};
//
//         #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display, EnumString)]
//         pub enum SDDecl {
//             #[strum(to_string=" standalone=\"yes\"", serialize="yes")]
//             Yes,
//             #[strum(to_string=" standalone=\"no\"", serialize="no")]
//             No
//         }
//
//         pub fn sd_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, SDDecl, E> {
//             preceded(
//                 delimited(multispace1, tag("standalone"), eq),
//                 alt((
//                     delimited(
//                         char::<I, E>('"'),
//                         map_res(is_not("\""), |value: I| value.as_ref().parse()),
//                         char::<I, E>('"'),
//                     ),
//                     delimited(
//                         char::<I, E>('\''),
//                         map_res(is_not("'"), |value: I| value.as_ref().parse()),
//                         char::<I, E>('\''),
//                     )
//                 ))
//             )(s)
//         }
//     }
//
//     mod white_space_handling {
//         // see https://www.w3.org/TR/REC-xml/#sec-white-space
//     }
//
//     mod end_of_line_handling {
//         use super::super::XMLParseError as ParseError;
//         use itertools::Itertools;
//         use memchr::memchr2_iter;
//         use nom::{AsBytes, Parser};
//         use std::borrow::Cow;
//
//         /// see: https://www.w3.org/TR/REC-xml/#sec-line-ends
//         pub fn normalize_newlines(s: &str) -> Cow<str> {
//             let found = memchr2_iter(b'\r', b'\n', s.as_bytes()).collect_vec();
//             if found.is_empty() {
//                 Cow::Borrowed(s)
//             } else {
//                 let mut last_index = None;
//                 let bytes = s.as_bytes();
//
//                 enum Instruction {
//                     Replace,
//                     Remove,
//                 }
//
//                 let mut instructions = Vec::with_capacity(found.len());
//                 let mut new_capacity = bytes.len();
//                 for index in found.into_iter() {
//                     if let Some(last) = last_index.replace(index) {
//                         if last + 1 == index {
//                             new_capacity -= 1;
//                             instructions.push((index, Instruction::Remove));
//                             continue
//                         }
//                     }
//                     let current = unsafe{*bytes.get_unchecked(index)};
//                     if b'\r' == current {
//                         instructions.push((index, Instruction::Replace));
//                     }
//                 }
//                 if instructions.is_empty() {
//                     Cow::Borrowed(s)
//                 } else {
//                     let mut new_string_content = Vec::with_capacity(new_capacity);
//                     let mut last_idx = 0usize;
//                     for (idx, instruction) in instructions.into_iter() {
//                         match instruction {
//                             Instruction::Replace => {
//                                 new_string_content.extend_from_slice(&bytes[last_idx..idx]);
//                                 new_string_content.push(b'\n');
//                                 last_idx = idx + 1;
//                             }
//                             Instruction::Remove => {
//                                 last_idx += 1;
//                             }
//                         }
//                     }
//                     Cow::Owned(String::from_utf8(new_string_content).unwrap())
//                 }
//             }
//         }
//
//         #[cfg(test)]
//         mod test_normalize_newlines {
//             use super::normalize_newlines;
//             use std::borrow::Cow;
//
//             const TEST_1: &str = "Hello world, this text does not cause an allocation very good!";
//             const TEST_2: &str = "Hello world, this text \n does not cause an allocation \n very good!";
//             const TEST_3: &str = "Hello \r world, this text \n does not cause an \r allocation \n very good!";
//             const TEST_3_EXP: &str = "Hello \n world, this text \n does not cause an \n allocation \n very good!";
//             const TEST_4: &str = "Hello \r\n world, this text \n does not cause an \r\n allocation \n very good!";
//             const TEST_4_EXP: &str = "Hello \n world, this text \n does not cause an \n allocation \n very good!";
//             const TEST_5: &str = "Hello \r\n world, this text \n\n\n\n does not cause an \r\n\r\n\r\n allocation \n very good!";
//             const TEST_5_EXP: &str = "Hello \n world, this text \n does not cause an \n allocation \n very good!";
//
//             #[test]
//             fn does_not_allocate_1() {
//                 assert!(matches!(normalize_newlines(TEST_1), Cow::Borrowed(_)))
//             }
//
//             #[test]
//             fn does_not_allocate_2() {
//                 assert!(matches!(normalize_newlines(TEST_2), Cow::Borrowed(_)))
//             }
//
//             #[test]
//             fn does_replace_carriage_returns() {
//                 let processed = normalize_newlines(TEST_3);
//                 assert!(matches!(processed, Cow::Owned(_)));
//                 assert_eq!(TEST_3_EXP, processed.as_ref());
//             }
//
//             #[test]
//             fn does_replace_carriage_return_and_line_feed() {
//                 let processed = normalize_newlines(TEST_4);
//                 assert!(matches!(processed, Cow::Owned(_)));
//                 assert_eq!(TEST_4_EXP, processed.as_ref());
//             }
//
//             #[test]
//             fn does_replace_multiple_carriage_returns_and_line_feeds() {
//                 let processed = normalize_newlines(TEST_5);
//                 assert!(matches!(processed, Cow::Owned(_)));
//                 assert_eq!(TEST_5_EXP, processed.as_ref());
//             }
//         }
//     }
//
//     mod language_identification {
//         // see https://www.w3.org/TR/REC-xml/#sec-lang-tag
//     }
//
//     // custom
//
//     mod cardinality {
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use nom::bytes::complete::take;
//         use nom::combinator::map_res;
//         use nom::IResult;
//         use std::str::FromStr;
//         use strum::{Display, EnumString};
//
//         #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Display, EnumString)]
//         pub enum Cardinality {
//             #[strum(to_string="?")]
//             ZeroOrOne,
//             #[strum(to_string="*")]
//             ZeroOrMany,
//             #[strum(to_string="+")]
//             OneOrMany,
//         }
//
//         pub fn cardinality<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Cardinality, E> {
//             map_res(take(1usize), |v: I| Cardinality::from_str(v.as_ref()))(s)
//         }
//     }
// }
//
// mod logical_structures {
//     pub use attribute_list_declarations::*;
//     pub use conditional_sections::*;
//     pub use element_content::*;
//     pub use element_type_declarations::*;
//     pub use mixed_content::*;
//     pub use start_tags_end_tags_and_empty_tags::*;
//
//     use super::XMLParseError as ParseError;
//     use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//     use nom::branch::alt;
//     use nom::combinator::map;
//     use nom::sequence::tuple;
//     use nom::IResult;
//     use strum::Display;
//     use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//     #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
//     pub enum Element<I> {
//         #[strum(to_string = "{0}")]
//         EmptyElementTag(EmptyElementTag<I>),
//         #[strum(to_string = "{0}{1}{2}")]
//         Element(STag<I>, Content<I>, ETag<I>)
//     }
//
//     impl<I> ToOwned2 for Element<I> where I: ToOwned {
//         type Owned2 = Element<I::Owned>;
//
//         fn to_owned2(&self) -> Self::Owned2 {
//             match self {
//                 Element::EmptyElementTag(value) => {
//                     Element::EmptyElementTag(value.to_owned2())
//                 }
//                 Element::Element(a, b, c) => {
//                     Element::Element(a.to_owned2(), b.to_owned2(), c.to_owned2())
//                 }
//             }
//         }
//     }
//
//     pub fn element<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Element<I>, E> {
//         alt((
//             map(empty_element_tag, Element::EmptyElementTag),
//             map(tuple((s_tag, content, e_tag)), |(a, b, c)| Element::Element(a, b, c)),
//         ))(s)
//     }
//
//
//     mod start_tags_end_tags_and_empty_tags {
//         use super::super::documents::{att_value, cd_sect, char_data, comment, eq, name, pi, AttValue, CDSect, CharData, Comment, Name, PI};
//         use super::super::physical_structures::{reference, Reference};
//         use super::super::XMLParseError as ParseError;
//         use super::{element, Element};
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use derive_more::From;
//         use nom::branch::alt;
//         use nom::bytes::complete::tag;
//         use nom::character::complete::{char, multispace0, multispace1};
//         use nom::combinator::{into, map, opt};
//         use nom::multi::many1;
//         use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use itertools::Itertools;
//         use strum::Display;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct STag<I>(pub Name<I>, pub Option<Vec<Attribute<I>>>);
//
//         impl<I> ToOwned2 for STag<I> where I: ToOwned {
//             type Owned2 = STag<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 STag(
//                     self.0.to_owned2(),
//                     self.1.as_ref().map(|value| value.iter().map(ToOwned2::to_owned2).collect_vec())
//                 )
//             }
//         }
//
//         impl<I> Display for STag<I> where I: Display + AsRef<str> {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<{}", self.0)?;
//                 if let Some(ref value) = self.1 {
//                     for v in value.iter() {
//                         write!(f, " {}", v)?;
//                     }
//                 }
//                 write!(f, ">")
//             }
//         }
//
//         pub fn s_tag<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, STag<I>, E> {
//             map(
//                 delimited(
//                     char('<'),
//                     pair(
//                         name,
//                         opt(many1(preceded(multispace1, attribute)))
//                     ),
//                     tag(">")
//                 ),
//                 |(a, b)| STag(a, b)
//             )(s)
//         }
//
//
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct Attribute<I>(pub Name<I>, pub AttValue<I>);
//
//         impl<I> ToOwned2 for Attribute<I> where I: ToOwned {
//             type Owned2 = Attribute<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Attribute(self.0.to_owned2(), self.1.to_owned2())
//             }
//         }
//
//         impl<I> Display for Attribute<I> where I: Display + AsRef<str> {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "{}={}", self.0, self.1)
//             }
//         }
//
//         pub fn attribute<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, Attribute<I>, E> {
//             map(
//                 separated_pair(
//                     name::<I, E>,
//                     eq::<I, E>,
//                     att_value::<I, E>
//                 ),
//                 |(a, b)| Attribute(a, b)
//             )(s)
//         }
//
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//         #[repr(transparent)]
//         pub struct ETag<I>(pub Name<I>);
//
//         impl<I> ToOwned2 for ETag<I> where I: ToOwned {
//             type Owned2 = ETag<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 ETag(self.0.to_owned2())
//             }
//         }
//
//         impl<I> Display for ETag<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "</{}>", self.0)
//             }
//         }
//
//         pub fn e_tag<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, ETag<I>, E> {
//             into(
//                 delimited(
//                     tag::<_, I, E>("</"),
//                     name::<I, E>,
//                     preceded(multispace0, char::<I, E>('>'))
//                 )
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//         pub enum InnerContent<I> {
//             #[strum(to_string="{0}")]
//             Element(Element<I>),
//             #[strum(to_string="{0}")]
//             Reference(Reference<I>),
//             #[strum(to_string="{0}")]
//             CDSect(CDSect<I>),
//             #[strum(to_string="{0}")]
//             PI(PI<I>),
//             #[strum(to_string="{0}")]
//             Comment(Comment<I>)
//         }
//
//         impl<I> ToOwned2 for InnerContent<I> where I: ToOwned {
//             type Owned2 = InnerContent<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     InnerContent::Element(value) => {
//                         InnerContent::Element(value.to_owned2())
//                     }
//                     InnerContent::Reference(value) => {
//                         InnerContent::Reference(value.to_owned2())
//                     }
//                     InnerContent::CDSect(value) => {
//                         InnerContent::CDSect(value.to_owned2())
//                     }
//                     InnerContent::PI(value) => {
//                         InnerContent::PI(value.to_owned2())
//                     }
//                     InnerContent::Comment(value) => {
//                         InnerContent::Comment(value.to_owned2())
//                     }
//                 }
//             }
//         }
//
//         fn inner_content<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, InnerContent<I>, E> {
//             alt((
//                 into(element::<_, E>),
//                 into(reference::<_, E>),
//                 into(cd_sect::<_, E>),
//                 into(pi::<_, E>),
//                 into(comment::<_, E>),
//             ))(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct Content<I>(
//             pub Option<CharData<I>>,
//             pub Option<Vec<(InnerContent<I>, Option<CharData<I>>)>>
//         );
//
//         impl<I> ToOwned2 for Content<I> where I: ToOwned {
//             type Owned2 = Content<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Content(
//                     self.0.as_ref().map(ToOwned2::to_owned2),
//                     self.1.as_ref().map(|value| {
//                         value.iter().map(|(a, b)| {
//                             (a.to_owned2(), b.as_ref().map(ToOwned2::to_owned2))
//                         }).collect_vec()
//                     })
//                 )
//             }
//         }
//
//         impl<I> Display for Content<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 if let Some(ref a) = self.0 {
//                     write!(f, "{a}")?;
//                 }
//                 if let Some(ref dat) = self.1 {
//                     for (ref i, ref v) in dat.iter(){
//                         write!(f, "{i}")?;
//                         if let Some(v) = v {
//                             write!(f, "{v}")?;
//                         }
//                     }
//                 }
//                 Ok(())
//             }
//         }
//
//         pub fn content<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, Content<I>, E> {
//             map(
//                 tuple((
//                     opt(char_data::<I, E>),
//                     opt(
//                         many1(
//                             pair(
//                                 inner_content::<I, E>,
//                                 opt(char_data::<I, E>)
//                             )
//                         )
//                     )
//                 )),
//                 |(a, b)| Content(a, b)
//             )(s)
//         }
//
//
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct EmptyElementTag<I>(
//             pub Name<I>,
//             pub Option<Vec<Attribute<I>>>
//         );
//
//         impl<I> ToOwned2 for EmptyElementTag<I> where I: ToOwned {
//             type Owned2 = EmptyElementTag<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 EmptyElementTag(
//                     self.0.to_owned2(),
//                     self.1.as_ref().map(|value| value.iter().map(ToOwned2::to_owned2).collect_vec())
//                 )
//             }
//         }
//
//         impl<I> Display for EmptyElementTag<I> where I: Display + AsRef<str> {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<{}", self.0)?;
//                 if let Some(ref value) = self.1 {
//                     for v in value.iter() {
//                         write!(f, " {}", v)?;
//                     }
//                 }
//                 write!(f, "/>")
//             }
//         }
//
//         pub fn empty_element_tag<I: DtdParserInput, E:ParseError<I>>(s: I) -> IResult<I, EmptyElementTag<I>, E> {
//             map(
//                 delimited(
//                     char::<I, E>('<'),
//                     pair(
//                         name::<I, E>,
//                         opt(many1(preceded(multispace1, attribute)))
//                     ),
//                     tag::<_, I, E>("/>")
//                 ),
//                 |(a, b)| EmptyElementTag(a, b)
//             )(s)
//         }
//     }
//
//     mod element_type_declarations {
//         use std::borrow::Borrow;
//         use super::super::documents::{name, Name};
//         use super::super::XMLParseError as ParseError;
//         use super::element_content::{children, Children};
//         use super::mixed_content::{mixed, Mixed};
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::{DtdParserInput, Merge};
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, MayBeUnresolved, MayBeUnresolvedRepr, ToOwned2};
//         use nom::branch::alt;
//         use nom::bytes::complete::tag;
//         use nom::character::complete::char as char2;
//         use nom::character::complete::{multispace0, multispace1};
//         use nom::combinator::{map, value};
//         use nom::sequence::{delimited, preceded, separated_pair, terminated};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use strum::Display;
//         use crate::topicmodel::dictionary::loader::dtd_parser::solving::{DTDResolver, ResolvableValue};
//
//         #[derive_where::derive_where(Debug; I: std::fmt::Debug)]
//         #[derive_where(Clone; I: Clone)]
//         #[derive_where(Eq; I: Eq)]
//         #[derive_where(PartialEq; I: PartialEq)]
//         #[derive_where(Hash; I: std::hash::Hash)]
//         pub struct ElementDecl<I>(pub Name<I>, pub MayBeUnresolved<I, ContentSpec<I>>);
//
//         impl<I> ToOwned2 for ElementDecl<I> where I: ToOwned {
//             type Owned2 = ElementDecl<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 ElementDecl(self.0.to_owned2(), self.1.to_owned2())
//             }
//         }
//
//         impl<I> Display for ElementDecl<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<!ELEMENT {} {}>\n", self.0, self.1)
//             }
//         }
//
//         pub fn element_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ElementDecl<I>, E> {
//             map(
//                 delimited(
//                     preceded(tag("<!ELEMENT"), multispace1),
//                     separated_pair(
//                         name,
//                         multispace1,
//                         may_be_unresolved(content_spec)
//                     ),
//                     terminated(multispace0, char2('>'))
//                 ),
//                 |(a, b)| ElementDecl(a, b)
//             )(s)
//         }
//
//         impl<I> ElementDecl<I>
//         where
//             I: ResolvableValue + ToOwned,
//             <I as Merge<I>>::Merged: Merge
//             + Merge<I, Merged=<I as Merge<I>>::Merged>
//             + Merge<char, Merged=<I as Merge<I>>::Merged>
//             + Merge<Merged=<I as Merge<I>>::Merged>,
//             char: Into<<I as Merge<I>>::Merged>,
//             <I as ToOwned>::Owned:
//                 for<'a> From<&'a str>
//                 + Merge<<I as ToOwned>::Owned, Merged=<I as ToOwned>::Owned>,
//         {
//             pub fn resolve<U>(&self, resolver: DTDResolver<I>) -> ElementDecl<I::Owned>
//             where
//                 <I as Merge<I>>::Merged: Borrow<U>,
//                 U: DtdParserInput + ToOwned<Owned=I::Owned>
//             {
//                 let name = self.0.to_owned2();
//                 match self.1.as_ref() {
//                     MayBeUnresolvedRepr::Resolved(value) => {
//                         todo!()
//                     }
//                     MayBeUnresolvedRepr::Unresolved(value) => {
//                         match resolver.resolve(value) {
//                             Some(value) => {
//                                 MayBeUnresolved::resolved(
//                                     content_spec(Borrow::<U>::borrow(value)).unwwrap().to_owned2()
//                                 )
//                             }
//                             None => {
//                                 MayBeUnresolved::unresolved(value.to_owned2())
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//
//         #[derive(Debug, Clone, Hash, PartialEq, Eq, Display)]
//         pub enum ContentSpec<I> {
//             #[strum(to_string="EMPTY")]
//             Empty,
//             #[strum(to_string="ANY")]
//             Any,
//             #[strum(to_string="{0}")]
//             Mixed(Mixed<I>),
//             #[strum(to_string="{0}")]
//             Children(Children<I>),
//         }
//
//
//         impl<I> ToOwned2 for ContentSpec<I> where I: ToOwned {
//             type Owned2 = ContentSpec<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     ContentSpec::Empty => {
//                         ContentSpec::Empty
//                     }
//                     ContentSpec::Any => {
//                         ContentSpec::Any
//                     }
//                     ContentSpec::Mixed(value) => {
//                         ContentSpec::Mixed(value.to_owned2())
//                     }
//                     ContentSpec::Children(value) => {
//                         ContentSpec::Children(value.to_owned2())
//                     }
//                 }
//             }
//         }
//
//
//         pub fn content_spec<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ContentSpec<I>, E> {
//             alt((
//                 value(ContentSpec::Empty, tag("EMPTY")),
//                 value(ContentSpec::Any, tag("ANY")),
//                 map(mixed, ContentSpec::Mixed),
//                 map(children, ContentSpec::Children),
//             ))(s)
//         }
//     }
//
//     mod element_content {
//         use super::super::documents::{cardinality, name, Cardinality, Name};
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, MayBeUnresolved, ToOwned2};
//         use derive_more::From;
//         use itertools::Itertools;
//         use nom::branch::alt;
//         use nom::character::complete::{char, multispace0};
//         use nom::combinator::{into, map, opt, verify};
//         use nom::multi::separated_list1;
//         use nom::sequence::{delimited, pair, preceded, terminated};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use strum::Display;
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//         pub enum InnerChildren<I> {
//             #[strum(to_string = "{0}")]
//             Choice(Choice<I>),
//             Seq(Seq<I>)
//         }
//
//         impl<I> ToOwned2 for InnerChildren<I> where I: ToOwned {
//             type Owned2 = InnerChildren<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     InnerChildren::Choice(value) => {
//                         InnerChildren::Choice(value.to_owned2())
//                     }
//                     InnerChildren::Seq(value) => {
//                         InnerChildren::Seq(value.to_owned2())
//                     }
//                 }
//             }
//         }
//
//         fn inner_child<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, InnerChildren<I>, E> {
//             alt((
//                 into(choice::<_, E>),
//                 into(seq::<_, E>),
//             ))(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct Children<I>(
//             pub InnerChildren<I>,
//             pub Option<Cardinality>
//         );
//
//         impl<I> ToOwned2 for Children<I> where I: ToOwned {
//             type Owned2 = Children<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Children(self.0.to_owned2(), self.1.clone())
//             }
//         }
//
//         impl<I> Display for Children<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "{}", self.0)?;
//                 if let Some(mu) = self.1 {
//                     write!(f, "{}", mu)?;
//                 }
//                 Ok(())
//             }
//         }
//
//         pub fn children<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Children<I>, E> {
//             map(
//                 pair(
//                     inner_child,
//                     opt(cardinality)
//                 ),
//                 |(a, b)| Children(a, b)
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//         pub enum CPInner<I> {
//             #[strum(to_string = "{0}")]
//             Name(Name<I>),
//             #[strum(to_string = "{0}")]
//             Choice(Choice<I>),
//             #[strum(to_string = "{0}")]
//             Seq(Seq<I>),
//         }
//
//         impl<I> ToOwned2 for CPInner<I> where I: ToOwned {
//             type Owned2 = CPInner<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     CPInner::Name(value) => {
//                         CPInner::Name(value.to_owned2())
//                     }
//                     CPInner::Choice(value) => {
//                         CPInner::Choice(value.to_owned2())
//                     }
//                     CPInner::Seq(value) => {
//                         CPInner::Seq(value.to_owned2())
//                     }
//                 }
//             }
//         }
//
//         fn cp_inner<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, CPInner<I>, E> {
//             alt((
//                 into(name::<_, E>),
//                 into(choice::<_, E>),
//                 into(seq::<_, E>),
//             ))(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct CP<I>(pub MayBeUnresolved<I, CPInner<I>>, pub Option<Cardinality>);
//
//         impl<I> ToOwned2 for CP<I> where I: ToOwned {
//             type Owned2 = CP<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 CP(self.0.to_owned2(), self.1.clone())
//             }
//         }
//
//         impl<I> Display for CP<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "{}", self.0)?;
//                 if let Some(c) = self.1 {
//                     write!(f, "{c}")?;
//                 }
//                 Ok(())
//             }
//         }
//
//         pub fn cp<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, CP<I>, E> {
//             map(
//                 pair(
//                     may_be_unresolved(cp_inner),
//                     opt(cardinality)
//                 ),
//                 |(a, b)| CP(a, b)
//             )(s)
//         }
//
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//         #[repr(transparent)]
//         pub struct Choice<I>(pub Vec<CP<I>>);
//
//         impl<I> ToOwned2 for Choice<I> where I: ToOwned {
//             type Owned2 = Choice<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Choice(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//             }
//         }
//
//
//         impl<I> Display for Choice<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "({})", self.0.iter().join("|"))
//             }
//         }
//
//         pub fn choice<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Choice<I>, E> {
//             into(
//                 delimited(
//                     terminated(char::<I, E>('('), multispace0),
//                     verify(
//                         separated_list1(
//                             delimited(multispace0, char::<I, E>('|'), multispace0),
//                             cp
//                         ),
//                         |value: &Vec<CP<I>>| { value.len() > 1 }
//                     ),
//                     preceded(multispace0, char::<I, E>(')'))
//                 )
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//         #[repr(transparent)]
//         pub struct Seq<I>(pub Vec<CP<I>>);
//
//         impl<I> ToOwned2 for Seq<I> where I: ToOwned {
//             type Owned2 = Seq<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Seq(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//             }
//         }
//
//         impl<I> Display for Seq<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "({})", self.0.iter().join(","))
//             }
//         }
//
//         pub fn seq<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Seq<I>, E> {
//             into(
//                 delimited(
//                     terminated(char::<I, E>('('), multispace0),
//                     separated_list1(
//                         delimited(multispace0, char::<I, E>(','), multispace0),
//                         cp
//                     ),
//                     preceded(multispace0, char::<I, E>(')'))
//                 )
//             )(s)
//         }
//     }
//
//     mod mixed_content {
//         use super::super::documents::{name, Name};
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, MayBeUnresolved, ToOwned2};
//         use nom::branch::alt;
//         use nom::bytes::complete::tag;
//         use nom::character::complete::{char, multispace0};
//         use nom::combinator::{map, value};
//         use nom::multi::many1;
//         use nom::sequence::{delimited, preceded, terminated, tuple};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use itertools::Itertools;
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         #[repr(transparent)]
//         pub struct Mixed<I>(pub Option<Vec<MayBeUnresolved<I, Name<I>>>>);
//
//         impl<I> ToOwned2 for Mixed<I> where I: ToOwned {
//             type Owned2 = Mixed<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Mixed(
//                     self.0
//                         .as_ref()
//                         .map(|value| value.iter().map(ToOwned2::to_owned2).collect_vec())
//                 )
//             }
//         }
//
//         impl<I> Display for Mixed<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "(#PCDATA")?;
//                 if let Some(ref names) = self.0 {
//                     for v in names {
//                         write!(f, "|  {v}")?;
//                     }
//                 }
//                 write!(f, ")")
//             }
//         }
//
//         pub fn mixed<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Mixed<I>, E> {
//             map(
//                 alt((
//                     delimited(
//                         tuple((char('('), multispace0, tag("#PCDATA"))),
//                         map(
//                             many1(
//                                 preceded(
//                                     delimited(multispace0, char('|'), multispace0),
//                                     may_be_unresolved(name)
//                                 ),
//                             ),
//                             Some
//                         ),
//                         preceded(multispace0, tag(")*"))
//                     ),
//                     value(
//                         None,
//                         delimited(
//                             terminated(char('('), multispace0),
//                             tag("#PCDATA"),
//                             preceded(multispace0, char(')'))
//                         )
//                     )
//                 )),
//                 Mixed
//             )(s)
//         }
//     }
//
//     mod attribute_list_declarations {
//         use super::super::documents::{name, Name};
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, may_be_unresolved_wrapped, MayBeUnresolved, ToOwned2};
//         use attribute_defaults::{default_decl, DefaultDecl};
//         use attribute_types::{att_type, AttType};
//         use itertools::Itertools;
//         use nom::bytes::complete::tag;
//         use nom::character::complete::{char, multispace0, multispace1};
//         use nom::combinator::{map, opt};
//         use nom::multi::many1;
//         use nom::sequence::{delimited, pair, preceded, terminated, tuple};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use std::str::FromStr;
//
//         // todo: https://www.w3.org/TR/REC-xml/#AVNormalize
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct AttlistDecl<I>(
//             pub Name<I>,
//             pub Option<Vec<MayBeUnresolved<I, AttDef<I>>>>
//         );
//
//         impl<I> ToOwned2 for AttlistDecl<I> where I: ToOwned {
//             type Owned2 = AttlistDecl<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 AttlistDecl(
//                     self.0.to_owned2(),
//                     self.1.as_ref().map(|value| value.iter().map(ToOwned2::to_owned2).collect_vec())
//                 )
//             }
//         }
//
//         impl<I> Display for AttlistDecl<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<!ATTLIST {}", self.0)?;
//                 if let Some(ref att) = self.1 {
//                     write!(f, "{}", att.iter().join(""))?;
//                 }
//                 write!(f, ">\n")
//             }
//         }
//
//         pub fn attlist_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, AttlistDecl<I>, E> {
//             map(
//                 delimited(
//                     terminated(tag("<!ATTLIST"), multispace1),
//                     pair(name, opt(many1(may_be_unresolved_wrapped(att_def, |value| preceded(multispace1, value))))),
//                     preceded(multispace0, char('>'))
//                 ),
//                 |(a, b)| AttlistDecl(a, b)
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct AttDef<I>(
//             pub Name<I>,
//             pub MayBeUnresolved<I, AttType<I>>,
//             pub DefaultDecl<I>
//         );
//
//         impl<I> ToOwned2 for AttDef<I> where I: ToOwned {
//             type Owned2 = AttDef<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 AttDef(
//                     self.0.to_owned2(),
//                     self.1.to_owned2(),
//                     self.2.to_owned2(),
//                 )
//             }
//         }
//
//         impl<I> Display for AttDef<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, " {} {} {}", self.0, self.1, self.2)
//             }
//         }
//
//         pub fn att_def<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, AttDef<I>, E> {
//             map(
//                 tuple((
//                     preceded(multispace1, name),
//                     preceded(multispace1, may_be_unresolved(att_type)),
//                     preceded(multispace1, default_decl),
//                 )),
//                 |(a, b, c)| AttDef(a, b, c)
//             )(s)
//         }
//
//         pub mod attribute_types {
//             use super::super::super::documents::{name, nm_token, Name, Nmtoken};
//             use super::super::super::XMLParseError as ParseError;
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use derive_more::From;
//             use itertools::Itertools;
//             use nom::branch::alt;
//             use nom::bytes::complete::tag;
//             use nom::character::complete::{alpha1, char, multispace0, multispace1};
//             use nom::combinator::{into, map, map_res, value};
//             use nom::multi::separated_list1;
//             use nom::sequence::{delimited, preceded, terminated, tuple};
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use std::str::FromStr;
//             use strum::{Display, EnumString};
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
//             pub enum AttType<I> {
//                 #[strum(to_string = "CDATA")]
//                 StringType,
//                 #[strum(to_string = "{0}")]
//                 TokenizedType(TokenizedType),
//                 #[strum(to_string = "{0}")]
//                 EnumeratedType(EnumeratedType<I>)
//             }
//
//             impl<I> ToOwned2 for AttType<I> where I: ToOwned {
//                 type Owned2 = AttType<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         AttType::StringType => {AttType::StringType}
//                         AttType::TokenizedType(value) => {AttType::TokenizedType(value.clone())}
//                         AttType::EnumeratedType(value) => {AttType::EnumeratedType(value.to_owned2())}
//                     }
//                 }
//             }
//
//             pub fn att_type<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, AttType<I>, E> {
//                 alt((
//                     value(AttType::StringType, tag("CDATA")),
//                     map(tokenized_type, AttType::TokenizedType),
//                     map(enumerated_type, AttType::EnumeratedType),
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display, EnumString)]
//             #[strum(serialize_all = "UPPERCASE")]
//             pub enum TokenizedType {
//                 Id,
//                 IdRef,
//                 IfRefs,
//                 Entity,
//                 Entities,
//                 NmToken,
//                 NmTokens
//             }
//
//             pub fn tokenized_type<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, TokenizedType, E> {
//                 map_res(alpha1, |v: I| TokenizedType::from_str(v.as_ref()))(s)
//             }
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//             pub enum EnumeratedType<I> {
//                 #[strum(to_string = "{0}")]
//                 NotationType(NotationType<I>),
//                 #[strum(to_string = "{0}")]
//                 Enumeration(Enumeration<I>)
//             }
//
//             impl<I> ToOwned2 for EnumeratedType<I> where I: ToOwned {
//                 type Owned2 = EnumeratedType<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         EnumeratedType::NotationType(value) => {EnumeratedType::NotationType(value.to_owned2())}
//                         EnumeratedType::Enumeration(value) => {EnumeratedType::Enumeration(value.to_owned2())}
//                     }
//                 }
//             }
//
//             pub fn enumerated_type<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EnumeratedType<I>, E> {
//                 alt((
//                     into(notation_type::<_, E>),
//                     into(enumeration::<_, E>),
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//             #[repr(transparent)]
//             pub struct NotationType<I>(pub Vec<Name<I>>);
//
//             impl<I> ToOwned2 for NotationType<I> where I: ToOwned {
//                 type Owned2 = NotationType<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     NotationType(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//                 }
//             }
//
//             impl<I> Display for NotationType<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "NOTATION ({})", self.0.iter().join("|"))
//                 }
//             }
//
//             pub fn notation_type<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, NotationType<I>, E> {
//                 into(
//                     delimited(
//                         tuple((tag::<_, I, E>("NOTATION"), multispace1, char('('), multispace0)),
//                         separated_list1(
//                             delimited(multispace0, char('|'), multispace0),
//                             name
//                         ),
//                         preceded(multispace0, char(')')),
//                     )
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//             #[repr(transparent)]
//             pub struct Enumeration<I>(pub Vec<Nmtoken<I>>);
//
//             impl<I> ToOwned2 for Enumeration<I> where I: ToOwned {
//                 type Owned2 = Enumeration<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     Enumeration(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//                 }
//             }
//
//             impl<I> Display for Enumeration<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "({})", self.0.iter().join("|"))
//                 }
//             }
//
//             pub fn enumeration<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Enumeration<I>, E> {
//                 into(
//                     delimited(
//                         preceded(char::<I, E>('('), multispace0),
//                         separated_list1(
//                             delimited(multispace0, char('|'), multispace0),
//                             nm_token
//                         ),
//                         terminated(multispace0, char(')')),
//                     )
//                 )(s)
//             }
//         }
//
//         pub mod attribute_defaults {
//             use super::super::super::documents::{att_value, AttValue};
//             use super::super::super::XMLParseError as ParseError;
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use nom::branch::alt;
//             use nom::bytes::complete::tag;
//             use nom::character::complete::multispace1;
//             use nom::combinator::{map, opt, value};
//             use nom::sequence::{preceded, terminated};
//             use nom::IResult;
//             use strum::Display;
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
//             pub enum DefaultDecl<I> {
//                 #[strum(to_string = "#REQUIRED")]
//                 Required,
//                 #[strum(to_string = "#IMPLIED")]
//                 Implied,
//                 #[strum(to_string = "#FIXED {0}")]
//                 AttValue(AttValue<I>)
//             }
//
//             impl<I> ToOwned2 for DefaultDecl<I> where I: ToOwned {
//                 type Owned2 = DefaultDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         DefaultDecl::Required => {
//                             DefaultDecl::Required
//                         }
//                         DefaultDecl::Implied => {
//                             DefaultDecl::Implied
//                         }
//                         DefaultDecl::AttValue(value) => {
//                             DefaultDecl::AttValue(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//             pub fn default_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, DefaultDecl<I>, E> {
//                 alt((
//                     value(DefaultDecl::Required, tag("#REQUIRED")),
//                     value(DefaultDecl::Implied, tag("#IMPLIED")),
//                     map(
//                         preceded(
//                             opt(terminated(tag("#FIXED"), multispace1)),
//                             att_value
//                         ),
//                         DefaultDecl::AttValue
//                     ),
//                 ))(s)
//             }
//         }
//
//         pub mod attribute_value_normalisation {
//             // todo: https://www.w3.org/TR/REC-xml/#AVNormalize
//         }
//     }
//
//     mod conditional_sections {
//         use super::super::documents::{ext_subset_decl, ExtSubsetDecl};
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use derive_more::From;
//         use itertools::Itertools;
//         use nom::branch::alt;
//         use nom::bytes::complete::{tag, take_until1};
//         use nom::character::complete::{char, multispace0};
//         use nom::combinator::{into, map, opt};
//         use nom::multi::{many0, many1};
//         use nom::sequence::{delimited, pair};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use strum::Display;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//         pub struct Ignore<I>(pub I);
//
//         impl<I> ToOwned2 for Ignore<I> where I: ToOwned {
//             type Owned2 = Ignore<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 Ignore(self.0.to_owned())
//             }
//         }
//
//         impl<I> Display for Ignore<I> where I: Display {
//             delegate::delegate! {
//                 to self.0 {
//                     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                 }
//             }
//         }
//
//         pub fn ignore<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Ignore<I>, E> {
//             into(
//                 nom::combinator::verify(
//                     take_until1::<_, I, E>("]]>"),
//                     |value: &I| !value.contains("<![")
//                 )
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//         pub struct IgnoreSectContents<I>(
//             pub Ignore<I>,
//             pub Option<Vec<(Box<IgnoreSectContents<I>>, Ignore<I>)>>
//         );
//
//         impl<I> ToOwned2 for IgnoreSectContents<I> where I: ToOwned {
//             type Owned2 = IgnoreSectContents<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 IgnoreSectContents(
//                     self.0.to_owned2(),
//                     self.1.as_ref().map(|value|{
//                     value.iter().map(
//                         |(a, b)|{
//                             (a.as_ref().to_owned2().into(), b.to_owned2())
//                         }
//                     ).collect_vec()
//                 }))
//             }
//         }
//
//         impl<I> Display for IgnoreSectContents<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "{}", self.0)?;
//                 if let Some(ref rest) = self.1 {
//                     for (cont, ign) in rest.iter(){
//                         write!(f, "<![{}]]>{}", cont, ign)?;
//                     }
//                 }
//                 Ok(())
//             }
//         }
//
//         pub fn ignore_sect_contents<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, IgnoreSectContents<I>, E> {
//             map(
//                 pair(
//                     ignore,
//                     opt(
//                         many1(
//                             pair(
//                                 map(
//                                     delimited(
//                                         tag("<!["),
//                                         ignore_sect_contents,
//                                         tag("]]>")
//                                     ),
//                                     Box::new
//                                 ),
//                                 ignore
//                             )
//                         )
//                     )
//                 ),
//                 |(a, b)| {
//                     IgnoreSectContents(a, b)
//                 }
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//         pub struct IgnoreSect<I>(pub Vec<IgnoreSectContents<I>>);
//
//         impl<I> ToOwned2 for IgnoreSect<I> where I: ToOwned {
//             type Owned2 = IgnoreSect<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 IgnoreSect(self.0.iter().map(ToOwned2::to_owned2).collect_vec())
//             }
//         }
//
//         impl<I> Display for IgnoreSect<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<![IGNORE[{}]]>", self.0.iter().join(""))
//             }
//         }
//
//         pub fn ignore_sect<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, IgnoreSect<I>, E> {
//             into(
//                 delimited(
//                     delimited(
//                         tag("<!["),
//                         delimited(
//                             multispace0::<I, E>,
//                             tag::<_, I, E>("IGNORE"),
//                             multispace0::<I, E>
//                         ),
//                         char('['),
//
//                     ),
//                     many0(ignore_sect_contents),
//                     tag("]]>")
//                 )
//             )(s)
//         }
//
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//         pub struct IncludeSect<I>(pub ExtSubsetDecl<I>);
//
//         impl<I> ToOwned2 for IncludeSect<I> where I: ToOwned {
//             type Owned2 = IncludeSect<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 IncludeSect(self.0.to_owned2())
//             }
//         }
//
//         impl<I> Display for IncludeSect<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<![INCLUDE[{}]]>", self.0)
//             }
//         }
//
//         pub fn include_sect<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, IncludeSect<I>, E> {
//             into(
//                 delimited(
//                     delimited(
//                         tag("<!["),
//                         delimited(
//                             multispace0::<I, E>,
//                             tag("INCLUDE"),
//                             multispace0
//                         ),
//                         char('['),
//
//                     ),
//                     ext_subset_decl,
//                     tag("]]>")
//                 )
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
//         pub enum ConditionalSect<I> {
//             #[strum(to_string="{0}")]
//             IncludeSect(IncludeSect<I>),
//             #[strum(to_string="{0}")]
//             IgnoreSect(IgnoreSect<I>)
//         }
//
//         impl<I> ToOwned2 for ConditionalSect<I> where I: ToOwned {
//             type Owned2 = ConditionalSect<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     ConditionalSect::IncludeSect(value) => {
//                         ConditionalSect::IncludeSect(value.to_owned2())
//                     }
//                     ConditionalSect::IgnoreSect(value) => {
//                         ConditionalSect::IgnoreSect(value.to_owned2())
//                     }
//                 }
//             }
//         }
//
//         pub fn conditional_sect<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ConditionalSect<I>, E> {
//             alt((
//                 into(include_sect::<_, E>),
//                 into(ignore_sect::<_, E>),
//             ))(s)
//         }
//     }
// }
//
// mod physical_structures {
//     pub use character_and_entity_references::*;
//     pub use entity_declarations::*;
//     pub use parsed_entities::*;
//
//     mod character_and_entity_references {
//         use super::super::documents::{name, Name};
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use derive_more::From;
//         use nom::branch::alt;
//         use nom::bytes::complete::tag;
//         use nom::character::complete::{char, digit1, hex_digit1};
//         use nom::combinator::{into, map_res};
//         use nom::sequence::delimited;
//         use nom::IResult;
//         use std::borrow::Borrow;
//         use std::fmt::{Display, Formatter};
//         use std::ops::Deref;
//         use strum::Display;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
//         #[repr(transparent)]
//         pub struct CharRef(pub char);
//
//         impl CharRef {
//             pub fn as_char(&self) -> char {
//                 self.0
//             }
//         }
//
//         impl Deref for CharRef {
//             type Target = char;
//
//             fn deref(&self) -> &Self::Target {
//                 &self.0
//             }
//         }
//
//         impl Display for CharRef {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "&#{};", self.0 as u32)
//             }
//         }
//
//         pub fn char_ref<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, CharRef, E> {
//             map_res(
//                 alt((
//                     delimited(
//                         tag::<_, I, E>("&#"),
//                         map_res(digit1, |value: I| u32::from_str_radix(value.as_ref(), 10)),
//                         char(';'),
//                     ),
//                     delimited(
//                         tag("&#x"),
//                         map_res(hex_digit1, |value: I| u32::from_str_radix(value.as_ref(), 16)),
//                         char(';'),
//                     )
//                 )),
//                 |value| {
//                     char::try_from(value).map(CharRef)
//                 }
//             )(s)
//         }
//
//
//         #[derive(Debug, Clone, Hash, PartialEq, Eq, Display, From)]
//         pub enum Reference<I> {
//             #[strum(to_string = "{0}")]
//             EntityRef(EntityRef<I>),
//             #[strum(to_string = "{0}")]
//             CharRef(CharRef),
//         }
//
//         impl<I> ToOwned2 for Reference<I> where I: ToOwned {
//             type Owned2 = Reference<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     Reference::EntityRef(value) => {
//                         Reference::EntityRef(value.to_owned2())
//                     }
//                     Reference::CharRef(value) => {
//                         Reference::CharRef(value.clone())
//                     }
//                 }
//             }
//         }
//
//
//         pub fn reference<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, Reference<I>, E> {
//             alt((
//                 into(char_ref::<_, E>),
//                 into(entity_ref::<_, E>),
//             ))(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//         #[repr(transparent)]
//         pub struct InnerEntityRef<I>(pub Name<I>);
//
//         impl<I> ToOwned2 for InnerEntityRef<I> where I: ToOwned {
//             type Owned2 = InnerEntityRef<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 InnerEntityRef(self.0.to_owned2())
//             }
//         }
//
//         impl<I> Borrow<Name<I>> for InnerEntityRef<I> {
//             fn borrow(&self) -> &Name<I> {
//                 &self.0
//             }
//         }
//
//         impl<I> Borrow<I> for InnerEntityRef<I> {
//             fn borrow(&self) -> &I {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> Deref for InnerEntityRef<I> where I: AsRef<str> {
//             type Target = str;
//
//             fn deref(&self) -> &Self::Target {
//                 &self.0
//             }
//         }
//
//         impl<I> Display for InnerEntityRef<I> where I: Display {
//             delegate::delegate! {
//                 to self.0 {
//                     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
//                 }
//             }
//         }
//
//         fn inner_entity_ref<I: DtdParserInput, E: ParseError<I>>(c: char) -> impl FnMut(I) -> IResult<I, InnerEntityRef<I>, E> {
//             into(
//                 delimited(
//                     char::<I, E>(c),
//                     name::<I, E>,
//                     char::<I, E>(';'),
//                 ),
//             )
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//         #[repr(transparent)]
//         pub struct EntityRef<I>(pub InnerEntityRef<I>);
//
//         impl<I> ToOwned2 for EntityRef<I> where I: ToOwned {
//             type Owned2 = EntityRef<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 EntityRef(self.0.to_owned2())
//             }
//         }
//
//         impl<I> Borrow<Name<I>> for EntityRef<I> {
//             fn borrow(&self) -> &Name<I> {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> Borrow<I> for EntityRef<I> {
//             fn borrow(&self) -> &I {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> AsRef<Name<I>> for EntityRef<I> {
//             fn as_ref(&self) -> &Name<I> {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> AsRef<I> for EntityRef<I> {
//             fn as_ref(&self) -> &I {
//                 self.0.borrow()
//             }
//         }
//
//
//         impl<I> Deref for EntityRef<I> where I: AsRef<str> {
//             type Target = str;
//
//             fn deref(&self) -> &Self::Target {
//                 &self.0
//             }
//         }
//
//         impl<I> Display for EntityRef<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "&{};", self.0)
//             }
//         }
//
//         #[inline(always)]
//         pub fn entity_ref<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EntityRef<I>, E> {
//             into(inner_entity_ref::<I, E>('&'))(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, From)]
//         #[repr(transparent)]
//         pub struct PEReference<I>(pub InnerEntityRef<I>);
//
//         impl<I> ToOwned2 for PEReference<I> where I: ToOwned {
//             type Owned2 = PEReference<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 PEReference(self.0.to_owned2())
//             }
//         }
//
//         impl<I> Deref for PEReference<I> where I: AsRef<str> {
//             type Target = str;
//
//             fn deref(&self) -> &Self::Target {
//                 &self.0
//             }
//         }
//
//         impl<I> Borrow<Name<I>> for PEReference<I> {
//             fn borrow(&self) -> &Name<I> {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> AsRef<Name<I>> for PEReference<I> {
//             fn as_ref(&self) -> &Name<I> {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> AsRef<I> for PEReference<I> {
//             fn as_ref(&self) -> &I {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> Borrow<I> for PEReference<I> {
//             fn borrow(&self) -> &I {
//                 self.0.borrow()
//             }
//         }
//
//         impl<I> Display for PEReference<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "%{};", self.0)
//             }
//         }
//
//         #[inline(always)]
//         pub fn pe_reference<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, PEReference<I>, E> {
//             into(inner_entity_ref::<I, E>('%'))(s)
//         }
//
//
//
//     }
//
//
//     mod entity_declarations {
//         use super::super::documents::{entity_value, name, EntityValue, Name};
//         use super::super::XMLParseError as ParseError;
//         use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//         use crate::topicmodel::dictionary::loader::dtd_parser::physical_structures::entity_declarations::external_entities::NDataDecl;
//         use derive_more::From;
//         pub use external_entities::*;
//         use nom::branch::alt;
//         use nom::bytes::complete::tag;
//         use nom::character::complete::{char, multispace0, multispace1};
//         use nom::combinator::{into, map, opt};
//         use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
//         use nom::IResult;
//         use std::fmt::{Display, Formatter};
//         use strum::Display;
//         use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//         pub enum EntityDecl<I> {
//             GEDecl(GEDecl<I>),
//             PEDecl(PEDecl<I>),
//         }
//
//         impl<I> ToOwned2 for EntityDecl<I> where I: ToOwned {
//             type Owned2 = EntityDecl<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     EntityDecl::GEDecl(value) => {
//                         EntityDecl::GEDecl(value.to_owned2())
//                     }
//                     EntityDecl::PEDecl(value) => {
//                         EntityDecl::PEDecl(value.to_owned2())
//                     }
//                 }
//             }
//         }
//
//         pub fn entity_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EntityDecl<I>, E> {
//             alt((
//                 into(ge_decl::<_, E>),
//                 into(pe_decl::<_, E>)
//             ))(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct GEDecl<I>(pub Name<I>, pub EntityDef<I>);
//
//         impl<I> ToOwned2 for GEDecl<I> where I: ToOwned {
//             type Owned2 = GEDecl<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 GEDecl(self.0.to_owned2(), self.1.to_owned2())
//             }
//         }
//
//         impl<I> Display for GEDecl<I> where I: Display + AsRef<str> {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<!ENTITY {} {}>\n", self.0, self.1)
//             }
//         }
//
//         pub fn ge_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, GEDecl<I>, E> {
//             map(
//                 delimited(
//                     terminated(tag("<!ENTITY"), multispace1),
//                     separated_pair(
//                         name,
//                         multispace1,
//                         entity_def
//                     ),
//                     preceded(multispace0, char('>'))
//                 ),
//                 |(a, b)| GEDecl(a, b)
//             )(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub struct PEDecl<I>(pub Name<I>, pub PEDef<I>);
//
//         impl<I> ToOwned2 for PEDecl<I> where I: ToOwned {
//             type Owned2 = PEDecl<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 PEDecl(self.0.to_owned2(), self.1.to_owned2())
//             }
//         }
//
//         impl<I> Display for PEDecl<I> where I: Display {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "<!ENTITY % {} {}>", self.0, self.1)
//             }
//         }
//
//         pub fn pe_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, PEDecl<I>, E> {
//             map(
//                 delimited(
//                     tuple((tag("<!ENTITY"), multispace1, char('%'), multispace1)),
//                     separated_pair(
//                         name,
//                         multispace1,
//                         pe_def
//                     ),
//                     preceded(multispace0, char('>'))
//                 ),
//                 |(a, b)| PEDecl(a, b)
//             )(s)
//         }
//
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//         pub enum EntityDef<I> {
//             EntityValue(EntityValue<I>),
//             ExternalId(ExternalID<I>, Option<NDataDecl<I>>)
//         }
//
//         impl<I> ToOwned2 for EntityDef<I> where I: ToOwned {
//             type Owned2 = EntityDef<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     EntityDef::EntityValue(value) => {
//                         EntityDef::EntityValue(value.to_owned2())
//                     }
//                     EntityDef::ExternalId(a, b) => {
//                         EntityDef::ExternalId(a.to_owned2(), b.as_ref().map(ToOwned2::to_owned2))
//                     }
//                 }
//             }
//         }
//
//
//         impl<I> Display for EntityDef<I> where I: Display + AsRef<str> {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 match self {
//                     EntityDef::EntityValue(value) => {
//                         Display::fmt(value, f)
//                     }
//                     EntityDef::ExternalId(id, data_decl) => {
//                         write!(f, "{id}")?;
//                         if let Some(data_decl) = data_decl {
//                             write!(f, "{data_decl}")?;
//                         }
//                         Ok(())
//                     }
//                 }
//             }
//         }
//
//         pub fn entity_def<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EntityDef<I>, E> {
//             alt((
//                 map(entity_value, EntityDef::EntityValue),
//                 map(pair(external_id, opt(n_data_decl)), |(a, b)| EntityDef::ExternalId(a, b)),
//             ))(s)
//         }
//
//         #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//         pub enum PEDef<I> {
//             #[strum(to_string = "{0}")]
//             EntityValue(EntityValue<I>),
//             #[strum(to_string = "{0}")]
//             ExternalId(ExternalID<I>)
//         }
//
//         impl<I> ToOwned2 for PEDef<I> where I: ToOwned {
//             type Owned2 = PEDef<I::Owned>;
//
//             fn to_owned2(&self) -> Self::Owned2 {
//                 match self {
//                     PEDef::EntityValue(value) => {
//                         PEDef::EntityValue(value.to_owned2())
//                     }
//                     PEDef::ExternalId(value) => {
//                         PEDef::ExternalId(value.to_owned2())
//                     }
//                 }
//             }
//         }
//
//         pub fn pe_def<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, PEDef<I>, E> {
//             alt((
//                 into(entity_value::<_, E>),
//                 into(external_id::<_, E>),
//             ))(s)
//         }
//
//
//         // internal_entities (Not needed)
//
//         pub mod external_entities {
//             use super::super::super::documents::{name, Name, PubidLiteral, SystemLiteral};
//             use super::super::super::XMLParseError as ParseError;
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use crate::topicmodel::dictionary::loader::dtd_parser::{pub_id_literal, system_literal};
//             use derive_more::From;
//             use nom::branch::alt;
//             use nom::bytes::complete::tag;
//             use nom::character::complete::multispace1;
//             use nom::combinator::{into, map};
//             use nom::sequence::{delimited, preceded, separated_pair, terminated};
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub struct ExternalID<I>(pub Option<PubidLiteral<I>>, pub SystemLiteral<I>);
//
//             impl<I> ToOwned2 for ExternalID<I> where I: ToOwned {
//                 type Owned2 = ExternalID<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     ExternalID(self.0.as_ref().map(ToOwned2::to_owned2), self.1.to_owned2())
//                 }
//             }
//
//             impl<I> ExternalID<I> {
//                 pub fn is_public(&self) -> bool {
//                     self.0.is_some()
//                 }
//             }
//
//             impl<I> Display for ExternalID<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     if let Some(ref pu) = self.0 {
//                         write!(f, "PUBLIC {} {}", pu, self.1)
//                     } else {
//                         write!(f, "SYSTEM {}", self.1)
//                     }
//                 }
//             }
//
//             pub fn external_id<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ExternalID<I>, E> {
//                 alt((
//                     map(preceded(terminated(tag("SYSTEM"), multispace1), system_literal),
//                         |value| ExternalID(None, value)
//                     ),
//                     map(
//                         preceded(terminated(tag("PUBLIC"), multispace1), separated_pair(pub_id_literal, multispace1, system_literal)),
//                         |(a, b)| {
//                             ExternalID(Some(a), b)
//                         }
//                     ),
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//             #[repr(transparent)]
//             pub struct NDataDecl<I>(pub Name<I>);
//
//             impl<I> ToOwned2 for NDataDecl<I> where I: ToOwned {
//                 type Owned2 = NDataDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     NDataDecl(self.0.to_owned2())
//                 }
//             }
//
//             impl<I> Display for NDataDecl<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, " NDATA {}", self.0)
//                 }
//             }
//
//             pub fn n_data_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, NDataDecl<I>, E> {
//                 into(
//                     preceded(
//                         delimited(multispace1, tag("NDATA"), multispace1),
//                         name::<I, E>
//                     )
//                 )(s)
//             }
//         }
//
//     }
//
//     mod parsed_entities {
//         pub use encoding_declaration::*;
//         pub use notation_declaration::*;
//         pub use text_declarations::*;
//         pub use well_formed_parsed_entities::*;
//
//         mod text_declarations {
//             use super::super::super::documents::{version_info, VersionInfo};
//             use super::super::super::XMLParseError as ParseError;
//             use super::{encoding_decl, EncodingDecl};
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use nom::bytes::complete::tag;
//             use nom::combinator::{map, opt};
//             use nom::sequence::{delimited, pair};
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub struct TextDecl<I>(pub EncodingDecl<I>, pub Option<VersionInfo<I>>);
//
//             impl<I> ToOwned2 for TextDecl<I> where I: ToOwned {
//                 type Owned2 = TextDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     TextDecl(self.0.to_owned2(), self.1.as_ref().map(ToOwned2::to_owned2))
//                 }
//             }
//
//             impl<I> Display for TextDecl<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "<?xml")?;
//                     if let Some(ref v) = self.1 {
//                         write!(f, "{v}")?;
//                     }
//                     write!(f, "{}?>", self.0)
//                 }
//             }
//
//             pub fn text_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, TextDecl<I>, E> {
//                 map(
//                     delimited(
//                         tag("<?xml"),
//                         pair(
//                             opt(version_info),
//                             encoding_decl
//                         ),
//                         tag("?>")
//                     ),
//                     |(a,b)| {
//                         TextDecl(b, a)
//                     }
//                 )(s)
//             }
//         }
//
//         mod well_formed_parsed_entities {
//             use super::super::super::logical_structures::{content, Content};
//             use super::super::super::XMLParseError as ParseError;
//             use super::{text_decl, TextDecl};
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use nom::combinator::{map, opt};
//             use nom::sequence::pair;
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//             pub struct ExtParsedEnt<I>(pub Option<TextDecl<I>>, pub Content<I>);
//
//             impl<I> ToOwned2 for ExtParsedEnt<I> where I: ToOwned {
//                 type Owned2 = ExtParsedEnt<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     ExtParsedEnt(self.0.as_ref().map(ToOwned2::to_owned2), self.1.to_owned2())
//                 }
//             }
//
//             impl<I> Display for ExtParsedEnt<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     if let Some(ref decl) = self.0 {
//                         write!(f, "{decl}")?;
//                     }
//                     write!(f, "{}", self.1)
//                 }
//             }
//
//             pub fn ext_parsed_ent<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, ExtParsedEnt<I>, E> {
//                 map(
//                     pair(
//                         opt(text_decl),
//                         content
//                     ),
//                     |(a, b)| ExtParsedEnt(a, b)
//                 )(s)
//             }
//         }
//
//         mod encoding_declaration {
//             use super::super::super::documents::eq;
//             use super::super::super::XMLParseError as ParseError;
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use derive_more::From;
//             use nom::branch::alt;
//             use nom::bytes::complete::{tag, take_while};
//             use nom::character::complete::{alpha1, char, multispace1};
//             use nom::combinator::{into, recognize};
//             use nom::sequence::{delimited, pair, preceded};
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//             #[repr(transparent)]
//             pub struct EncodingDecl<I>(pub EncName<I>);
//
//             impl<I> ToOwned2 for EncodingDecl<I> where I: ToOwned {
//                 type Owned2 = EncodingDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     EncodingDecl(self.0.to_owned2())
//                 }
//             }
//
//             impl<I> Display for EncodingDecl<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, " encoding=\"{}\"", self.0)
//                 }
//             }
//
//             pub fn encoding_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EncodingDecl<I>, E> {
//                 into(
//                     preceded(
//                         delimited(multispace1::<I, E>, tag("encoding"), eq),
//                         alt((
//                             delimited(
//                                 char('"'),
//                                 enc_name,
//                                 char('"'),
//                             ),
//                             delimited(
//                                 char('\''),
//                                 enc_name,
//                                 char('\''),
//                             )
//                         ))
//                     )
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//             #[repr(transparent)]
//             pub struct EncName<I>(pub I);
//
//             impl<I> ToOwned2 for EncName<I> where I: ToOwned {
//                 type Owned2 = EncName<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     EncName(self.0.to_owned())
//                 }
//             }
//
//             impl<I> Display for EncName<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "{}", self.0)
//                 }
//             }
//
//             pub fn enc_name<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, EncName<I>, E> {
//                 into(
//                     recognize(
//                         pair(
//                             alpha1::<I, E>,
//                             take_while(|value| {
//                                 nom::AsChar::is_alpha(value) || value == '.' || value == '_' || value == '-'
//                             })
//                         )
//                     ),
//                 )(s)
//             }
//         }
//
//         mod notation_declaration {
//             use super::super::super::documents::{name, pub_id_literal, Name, PubidLiteral};
//             use super::super::super::physical_structures::{external_id, ExternalID};
//             use super::super::super::XMLParseError as ParseError;
//             use crate::topicmodel::dictionary::loader::dtd_parser::input::DtdParserInput;
//             use derive_more::From;
//             use nom::branch::alt;
//             use nom::bytes::complete::tag;
//             use nom::character::complete::{char, multispace0, multispace1};
//             use nom::combinator::{into, map};
//             use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
//             use nom::IResult;
//             use std::fmt::{Display, Formatter};
//             use strum::Display;
//             use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
//             pub enum InnerNotationDeclId<I> {
//                 ExternalId(ExternalID<I>),
//                 PublicId(PublicID<I>),
//             }
//
//             impl<I> ToOwned2 for InnerNotationDeclId<I> where I: ToOwned {
//                 type Owned2 = InnerNotationDeclId<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     match self {
//                         InnerNotationDeclId::ExternalId(value) => {
//                             InnerNotationDeclId::ExternalId(value.to_owned2())
//                         }
//                         InnerNotationDeclId::PublicId(value) => {
//                             InnerNotationDeclId::PublicId(value.to_owned2())
//                         }
//                     }
//                 }
//             }
//
//             pub fn inner_notation_decl_id<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, InnerNotationDeclId<I>, E> {
//                 alt((
//                     into(external_id::<_, E>),
//                     into(public_id::<_, E>)
//                 ))(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq)]
//             pub struct NotationDecl<I>(pub Name<I>, pub InnerNotationDeclId<I>);
//
//             impl<I> ToOwned2 for NotationDecl<I> where I: ToOwned {
//                 type Owned2 = NotationDecl<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     NotationDecl(self.0.to_owned2(), self.1.to_owned2())
//                 }
//             }
//
//             impl<I> Display for NotationDecl<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "<!NOTATION {} {}>\n", self.0, self.1)
//                 }
//             }
//
//             pub fn notation_decl<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, NotationDecl<I>, E> {
//                 map(
//                     delimited(
//                         terminated(tag("<!NOTATION"), multispace1),
//                         separated_pair(
//                             name,
//                             multispace1,
//                             inner_notation_decl_id
//                         ),
//                         preceded(multispace0, char('>'))
//                     ),
//                     |(a, b)| NotationDecl(a, b)
//                 )(s)
//             }
//
//             #[derive(Debug, Clone, Hash, Eq, PartialEq, From)]
//             #[repr(transparent)]
//             pub struct PublicID<I>(pub PubidLiteral<I>);
//
//             impl<I> ToOwned2 for PublicID<I> where I: ToOwned {
//                 type Owned2 = PublicID<I::Owned>;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     PublicID(self.0.to_owned2())
//                 }
//             }
//
//             impl<I> Display for PublicID<I> where I: Display {
//                 fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                     write!(f, "PUBLIC {}", self.0)
//                 }
//             }
//
//             pub fn public_id<I: DtdParserInput, E: ParseError<I>>(s: I) -> IResult<I, PublicID<I>, E> {
//                 into(
//                     preceded(pair(tag("PUBLIC"), multispace1::<I, E>), pub_id_literal)
//                 )(s)
//             }
//         }
//     }
// }
//
//
//
// pub mod solving {
//     use std::borrow::Cow;
//     use std::collections::{HashMap, VecDeque};
//     use std::error::Error;
//     use std::hash::Hash;
//     use derive_more::From;
//     use itertools::{merge, Either};
//     use crate::topicmodel::dictionary::loader::dtd_parser::{CharRef, EntityDecl, EntityDef, EntityRef, EntityValue, EntityValuePart, Name, PEDef, PEReference, PublicID, Reference};
//     use crate::topicmodel::dictionary::loader::dtd_parser::input::{merge_into_opt, DtdParserInput, Merge};
//     use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::ToOwned2;
//
//     // todo: https://www.w3.org/TR/REC-xml/#intern-replacement
//     #[derive(Debug, Default)]
//     pub struct DTDResolver<T> {
//         resolved: HashMap<Name<T>, EntityValue<T>>,
//     }
//
//     impl<I> ToOwned2 for DTDResolver<I>
//     where
//         I: ToOwned,
//         <I as ToOwned>::Owned: Eq + Hash
//     {
//         type Owned2 = DTDResolver<I::Owned>;
//
//         fn to_owned2(&self) -> Self::Owned2 {
//             DTDResolver {
//                 resolved: self.resolved.iter().map(|(k, v)| {
//                     (k.to_owned2(), v.to_owned2())
//                 }).collect()
//             }
//         }
//     }
//
//     impl<T> DTDResolver<T> where T: Clone + Eq + Hash {
//         pub fn register(&mut self, entity_decl: &EntityDecl<T>) {
//             match entity_decl {
//                 EntityDecl::GEDecl(decl) => {
//                     match &decl.1 {
//                         EntityDef::EntityValue(value) => {
//                             self.resolved.insert(decl.0.clone(), value.clone());
//                         }
//                         EntityDef::ExternalId(_, _) => {
//                             log::warn!("ExternalID not supported!")
//                         }
//                     }
//                 }
//                 EntityDecl::PEDecl(decl) => {
//                     match &decl.1 {
//                         PEDef::EntityValue(value) => {
//                             self.resolved.insert(decl.0.clone(), value.clone());
//                         }
//                         PEDef::ExternalId(value) => {
//                             log::warn!("ExternalID not supported!")
//                         }
//                     }
//                 }
//             }
//         }
//     }
//
//     #[derive(Copy, Clone, Eq, PartialEq, Hash, From)]
//     pub enum ResolvableReference<'a, T> {
//         PEReference(&'a PEReference<T>),
//         EntityRef(&'a EntityRef<T>),
//         CharRef(CharRef),
//     }
//
//     #[derive(Copy, Clone, Eq, PartialEq, Hash, From)]
//     pub enum EitherReference<'a, T> {
//         PEReference(&'a PEReference<T>),
//         EntityRef(&'a EntityRef<T>),
//     }
//
//
//     impl<'a, T> From<&'a Reference<T>> for ResolvableReference<'a, T> {
//         fn from(value: &'a Reference<T>) -> Self {
//             match value {
//                 Reference::EntityRef(value) => {
//                     Self::EntityRef(value)
//                 }
//                 Reference::CharRef(value) => {
//                     Self::CharRef(value.clone())
//                 }
//             }
//         }
//     }
//
//     pub trait ResolvableValue: Clone + Eq + Hash + Merge<Self> + Into<<Self as Merge<Self>>::Merged>
//     where
//     <Self as Merge<Self>>::Merged: Merge<Self, Merged=<Self as Merge<Self>>::Merged> + Merge<char, Merged=<Self as Merge<Self>>::Merged> + Merge<Merged=<Self as Merge<Self>>::Merged>,
//     char: Into<<Self as Merge<Self>>::Merged>,
//     {}
//
//     impl<T> ResolvableValue for T where
//         T: Clone + Eq + Hash + Merge + Into<<T as Merge<T>>::Merged>,
//         <T as Merge<T>>::Merged: Merge
//         + Merge<T, Merged=<T as Merge<T>>::Merged>
//         + Merge<char, Merged=<T as Merge<T>>::Merged>
//         + Merge<Merged=<T as Merge<T>>::Merged>,
//         char: Into<<T as Merge<T>>::Merged>,
//     {}
//
//     impl<T> DTDResolver<T>
//     where
//         T: ResolvableValue,
//         <T as Merge<T>>::Merged: Merge
//         + Merge<T, Merged=<T as Merge<T>>::Merged>
//         + Merge<char, Merged=<T as Merge<T>>::Merged>
//         + Merge<Merged=<T as Merge<T>>::Merged>,
//         char: Into<<T as Merge<T>>::Merged>,
//     {
//         pub fn resolve<'a>(&self, to_resolve: impl Into<ResolvableReference<'a, T>>) -> Option<<T as Merge>::Merged> where T: 'a {
//             let mut merged: Option<<T as Merge<T>>::Merged> = None;
//             let mut queue = VecDeque::new();
//             queue.push_back(to_resolve.into());
//
//
//             while let Some(next) = queue.pop_front() {
//                 let resolved = match next {
//                     ResolvableReference::PEReference(value) => {
//                         if let Some(found) = self.resolved.get(AsRef::<T>::as_ref(value)) {
//                             Either::Left(found)
//                         } else {
//                             Either::Right(EitherReference::from(value))
//                         }
//                     }
//                     ResolvableReference::EntityRef(value) => {
//                         if let Some(found) = self.resolved.get(AsRef::<T>::as_ref(value)) {
//                             Either::Left(found)
//                         } else {
//                             Either::Right(EitherReference::from(value))
//                         }
//                     }
//                     ResolvableReference::CharRef(value) => {
//                         merged = if let Some(m) = merged {
//                             Some(Merge::<char>::merge(m, value.as_char()))
//                         } else {
//                             Some(value.0.into())
//                         };
//                         continue
//                     }
//                 };
//
//                 match resolved {
//                     Either::Left(found) => {
//                         for v in found.iter() {
//                             match v {
//                                 EntityValuePart::Raw(value) => {
//                                     merged = merge_into_opt(merged, value.clone());
//                                 }
//                                 EntityValuePart::PEReference(value) => {
//                                     queue.push_back(value.into());
//                                 }
//                                 EntityValuePart::Reference(value) => {
//                                     queue.push_back(value.into());
//                                 }
//                             }
//                         }
//                     }
//                     Either::Right(reference) => {
//                         let value: &T = match reference {
//                             EitherReference::PEReference(reference) => {
//                                 reference.as_ref()
//                             }
//                             EitherReference::EntityRef(reference) => {
//                                 reference.as_ref()
//                             }
//                         };
//
//                         merged = merge_into_opt(merged, value.clone());
//                     }
//                 }
//             }
//             merged
//         }
//     }
//
//
//     pub trait Resolvable<T>
//     where
//         T: ResolvableValue
//     {
//         type Resolved;
//
//         fn resolve<E: Error>(&self, resolver: &mut DTDResolver<T>) -> Result<Self::Resolved, E>
//         where
//             <T as Merge<T>>::Merged: Merge
//             + Merge<T, Merged=<T as Merge<T>>::Merged>
//             + Merge<char, Merged=<T as Merge<T>>::Merged>
//             + Merge<Merged=<T as Merge<T>>::Merged>,
//               char: Into<<T as Merge<T>>::Merged>,;
//     }
//
// }
//
// pub mod input {
//     use itertools::Itertools;
//     use nom::{Compare, FindSubstring, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset, Slice};
//     use std::fmt::{Debug, Display, Pointer};
//     use std::hash::Hash;
//     use std::ops::{RangeFrom, RangeTo};
//
//     pub trait Contains<T> {
//         fn contains(&self, other: T) -> bool;
//     }
//
//
//     pub trait Merge<TSelf = Self> {
//         type Merged;
//
//         fn merge(self, other: TSelf) -> Self::Merged;
//     }
//
//     pub trait DtdParserInput<TSelf = Self>
//     : InputTakeAtPosition<Item=char>
//     + Clone
//     + Offset
//     + Slice<RangeTo<usize>>
//     + Slice<RangeFrom<usize>>
//     + InputIter<Item=char>
//     + AsRef<str>
//     + InputLength
//     + InputTake
//     + Merge<TSelf>
//     + for<'a> Compare<&'a str>
//     + for<'a> FindSubstring<&'a str>
//     + for<'a> Contains<&'a str>
//     {}
//
//
//     pub fn merge_into_opt<T>(a: Option<T::Merged>, b: T) -> Option<T::Merged>
//     where
//         T: Merge<T> + Into<T::Merged>,
//         <T as Merge<T>>::Merged: Merge<Merged=<T as Merge<T>>::Merged>
//     {
//         match a {
//             None => {
//                 Some(b.into())
//             }
//             Some(value) => {
//                 Some(value.merge(b.into()))
//             }
//         }
//     }
//
//     impl<'a> Merge for &'a str {
//         type Merged = String;
//
//         fn merge(self, other: Self) -> Self::Merged {
//             let mut new = String::with_capacity(self.len() + other.len());
//             new.push_str(self);
//             new.push_str(other);
//             new
//         }
//     }
//
//     impl<'a> Merge<char> for &'a str {
//         type Merged = String;
//
//         fn merge(self, other: char) -> Self::Merged {
//             let mut new = String::with_capacity(self.len() + 1);
//             new.push_str(self);
//             new.push(other);
//             new
//         }
//     }
//
//     impl<'a> Merge for String {
//         type Merged = String;
//
//         fn merge(mut self, other: String) -> Self::Merged {
//             self.push_str(other.as_str());
//             self
//         }
//     }
//
//     impl<'a> Merge<&'a str> for String {
//         type Merged = String;
//
//         fn merge(mut self, other: &'a str) -> Self::Merged {
//             self.push_str(other);
//             self
//         }
//     }
//
//     impl Merge<char> for String {
//         type Merged = String;
//
//         fn merge(mut self, other: char) -> Self::Merged {
//             self.push(other);
//             self
//         }
//     }
//
//     pub trait SupportsMerged {
//         type Merged;
//
//         fn get_merged(&self) -> Self::Merged;
//     }
//
//     impl<'a, 'b> Contains<&'b str> for &'a str {
//         fn contains(&self, other: &'b str) -> bool {
//             str::contains(self, other)
//         }
//     }
//
//     impl<T> DtdParserInput<T> for T
//     where
//         T: InputTakeAtPosition<Item=char>
//         + Clone
//         + Offset
//         + Slice<RangeTo<usize>>
//         + Slice<RangeFrom<usize>>
//         + InputIter<Item=char>
//         + AsRef<str>
//         + InputLength
//         + InputTake
//         + Merge<T>
//         + for<'a> Compare<&'a str>
//         + for<'a> FindSubstring<&'a str>
//         + for<'a> Contains<&'a str>
//     {}
// }
//
// /// Because we are a primitive parser, we resolve everything after the fact
// pub mod unresolved_helper {
//     use crate::topicmodel::dictionary::loader::dtd_parser::errors::XMLParseError;
//     use crate::topicmodel::dictionary::loader::dtd_parser::input::{DtdParserInput, Merge};
//     use crate::topicmodel::dictionary::loader::dtd_parser::{entity_ref, pe_reference, EntityRef, Name, PEReference};
//     use derive_more::From;
//     use nom::branch::alt;
//     use nom::combinator::{into, map};
//     use nom::{IResult, Parser};
//     use std::borrow::Borrow;
//     use std::fmt::{Debug, Display, Formatter};
//     use std::marker::PhantomData;
//     use std::ops::Deref;
//     use itertools::Itertools;
//     use strum::Display;
//     use crate::topicmodel::dictionary::loader::dtd_parser::solving::ResolvableReference;
//
//     #[derive(Display, Debug, Clone)]
//     #[derive_where::derive_where(Eq; I: Eq, T: Eq)]
//     #[derive_where(PartialEq; I: PartialEq, T: PartialEq)]
//     #[derive_where(Hash; I: std::hash::Hash, T: std::hash::Hash)]
//     pub enum MayBeUnresolvedRepr<I, T> {
//         #[strum(to_string = "{0}")]
//         Resolved(T),
//         #[strum(to_string = "{0}")]
//         Unresolved(UnresolvedReference<I>)
//     }
//
//     impl<I, T> ToOwned2 for MayBeUnresolvedRepr<I, T> where I: ToOwned, T: ToOwned2 {
//         type Owned2 = MayBeUnresolvedRepr<I::Owned, T::Owned2>;
//
//         fn to_owned2(&self) -> Self::Owned2 {
//             match self {
//                 MayBeUnresolvedRepr::Resolved(value) => {
//                     MayBeUnresolvedRepr::Resolved(value.to_owned2())
//                 }
//                 MayBeUnresolvedRepr::Unresolved(value) => {
//                     MayBeUnresolvedRepr::Unresolved(value.to_owned2())
//                 }
//             }
//         }
//     }
//
//     #[derive_where::derive_where(Clone; I: Clone, T: Clone)]
//     #[derive_where(Debug; I: Debug, T: Debug)]
//     #[derive_where(Eq; I: Eq, T: Eq)]
//     #[derive_where(PartialEq; I: PartialEq, T: PartialEq)]
//     #[derive_where(Hash; I: std::hash::Hash, T: std::hash::Hash)]
//     pub struct MayBeUnresolved<I, T> {
//         inner: MayBeUnresolvedRepr<I, T>
//     }
//
//     impl<I, T> ToOwned2 for MayBeUnresolved<I, T> where I: ToOwned, T: ToOwned2 {
//         type Owned2 = MayBeUnresolved<I::Owned, T::Owned2>;
//
//         fn to_owned2(&self) -> Self::Owned2 {
//             MayBeUnresolved {
//                 inner: self.inner.to_owned2()
//             }
//         }
//     }
//
//     impl<I, T> MayBeUnresolved<I, T> {
//         pub fn is_unresolved(&self) -> bool {
//             matches!(self.inner, MayBeUnresolvedRepr::Unresolved(_))
//         }
//
//         pub fn as_resolved(&self) -> Option<&T> {
//             match self.inner {
//                 MayBeUnresolvedRepr::Resolved(ref value) => {
//                     Some(value)
//                 }
//                 MayBeUnresolvedRepr::Unresolved(_) => {
//                     None
//                 }
//             }
//         }
//
//         pub fn as_mut_resolved(&mut self) -> Option<&mut T> {
//             match &mut self.inner {
//                 MayBeUnresolvedRepr::Resolved(value) => {
//                     Some(value)
//                 }
//                 MayBeUnresolvedRepr::Unresolved(_) => {
//                     None
//                 }
//             }
//         }
//
//         pub fn as_unresolved(&self) -> Option<&UnresolvedReference<I>> {
//             match self.inner {
//                 MayBeUnresolvedRepr::Resolved(_) => {
//                     None
//                 }
//                 MayBeUnresolvedRepr::Unresolved(ref value) => {
//                     Some(value)
//                 }
//             }
//         }
//
//         pub fn set_resolved(&mut self, resolved: T) -> MayBeUnresolved<I, T> {
//             MayBeUnresolved {
//                 inner: std::mem::replace(&mut self.inner, MayBeUnresolvedRepr::Resolved(resolved))
//             }
//         }
//
//         pub fn unresolved(reference: UnresolvedReference<I>) -> Self{
//             Self {
//                 inner: MayBeUnresolvedRepr::Unresolved(reference)
//             }
//         }
//
//         pub fn resolved(value: T) -> Self{
//             Self {
//                 inner: MayBeUnresolvedRepr::Resolved(value)
//             }
//         }
//     }
//
//     impl<I, T> AsRef<MayBeUnresolvedRepr<I, T>> for MayBeUnresolved<I, T> {
//         fn as_ref(&self) -> &MayBeUnresolvedRepr<I, T> {
//             &self.inner
//         }
//     }
//
//     impl<I, T> AsMut<MayBeUnresolvedRepr<I, T>> for MayBeUnresolved<I, T> {
//         fn as_mut(&mut self) -> &mut MayBeUnresolvedRepr<I, T> {
//             &mut self.inner
//         }
//     }
//
//     impl<I, T> Display for MayBeUnresolved<I, T> where T: Display, I: Display {
//         fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//             Display::fmt(&self.inner, f)
//         }
//     }
//
//
//     pub struct UnresolvedReferenceFn<I, E> {
//         _dat: PhantomData<fn(I) -> E>
//     }
//
//     impl<I, E> UnresolvedReferenceFn<I, E> {
//         pub fn new() -> Self {
//             Self { _dat: PhantomData }
//         }
//     }
//
//     impl<I: DtdParserInput, E> Parser<I, UnresolvedReference<I>, E> for UnresolvedReferenceFn<I, E>
//         where
//             E: XMLParseError<I>
//     {
//         fn parse(&mut self, input: I) -> IResult<I, UnresolvedReference<I>, E> {
//             unresolved_reference(input)
//         }
//     }
//
//     /// Helps the parser to parse an element or attlist with somne unresolved data.
//     #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
//     pub enum UnresolvedReference<I> {
//         #[strum(to_string="{0}")]
//         EntityRef(EntityRef<I>),
//         #[strum(to_string="{0}")]
//         PEReference(PEReference<I>)
//     }
//
//     impl<'a, I> Into<ResolvableReference<'a, I>> for &'a UnresolvedReference<I> {
//         fn into(self) -> ResolvableReference<'a, I> {
//             match self {
//                 UnresolvedReference::EntityRef(value) => {
//                     value.into()
//                 }
//                 UnresolvedReference::PEReference(value) => {
//                     value.into()
//                 }
//             }
//         }
//     }
//
//     impl<I> ToOwned2 for UnresolvedReference<I> where I: ToOwned {
//         type Owned2 = UnresolvedReference<I::Owned>;
//
//         fn to_owned2(&self) -> Self::Owned2 {
//             match self {
//                 UnresolvedReference::EntityRef(value) => {
//                     UnresolvedReference::EntityRef(value.to_owned2())
//                 }
//                 UnresolvedReference::PEReference(value) => {
//                     UnresolvedReference::PEReference(value.to_owned2())
//                 }
//             }
//         }
//     }
//
//     impl<I> Borrow<Name<I>> for UnresolvedReference<I> {
//         fn borrow(&self) -> &Name<I> {
//             self.deref()
//         }
//     }
//
//     impl<I> Deref for UnresolvedReference<I> {
//         type Target = Name<I>;
//
//         fn deref(&self) -> &Self::Target {
//             match self {
//                 UnresolvedReference::EntityRef(a) => {a.borrow()}
//                 UnresolvedReference::PEReference(a) => {a.borrow()}
//             }
//         }
//     }
//
//     pub fn unresolved_reference<I: DtdParserInput, E: XMLParseError<I>>(s: I) -> IResult<I, UnresolvedReference<I>, E> {
//         alt((
//             into(entity_ref::<_, E>),
//             into(pe_reference::<_, E>),
//         ))(s)
//     }
//
//     pub fn may_be_unresolved<I: DtdParserInput, O1, E, F>(parser: F) -> impl FnMut(I) -> IResult<I, MayBeUnresolved<I, O1>, E>
//         where
//             F: Parser<I, O1, E>,
//             E: XMLParseError<I>
//     {
//         alt((
//             map(parser, MayBeUnresolved::resolved),
//             map(unresolved_reference, MayBeUnresolved::unresolved),
//         ))
//     }
//
//     pub fn may_be_unresolved_wrapped<I: DtdParserInput, O1, E, F, W, Q>(parser: F, wrapper: W) -> impl FnMut(I) -> IResult<I, MayBeUnresolved<I, O1>, E>
//     where
//         F: Parser<I, O1, E>,
//         W: Fn(UnresolvedReferenceFn<I, E>) -> Q,
//         Q: Parser<I, UnresolvedReference<I>, E>,
//         E: XMLParseError<I>
//     {
//         alt((
//             map(parser, MayBeUnresolved::resolved),
//             map(wrapper(UnresolvedReferenceFn::new()), MayBeUnresolved::unresolved),
//         ))
//     }
//
//
//     pub struct Resolved<D, T> {
//         data: D,
//         data_holder: T
//     }
//
//     impl<D, T> Resolved<D, T>
//
//     {
//         pub fn parse_as_resolved<I, F>(mut parser: F, ) -> Self
//         where
//             T: Merge<Merged=D>,
//             D: Sized + AsRef<I>,
//         {
//             todo!()
//         }
//     }
//
//
//     pub trait ToOwned2 {
//         type Owned2;
//
//         fn to_owned2(&self) -> Self::Owned2;
//     }
//
//
//     macro_rules! to_owned_mapping {
//         ($($v: ty),+) => {
//             $(
//             impl ToOwned2 for $v {
//                 type Owned2 = <$v as ToOwned>::Owned;
//
//                 fn to_owned2(&self) -> Self::Owned2 {
//                     self.to_owned()
//                 }
//             }
//             )+
//         };
//     }
//
//     to_owned_mapping!(str);
// }
//
//
// #[cfg(test)]
// mod test {
//     use crate::topicmodel::dictionary::loader::dtd_parser::doc_type_no_decl;
//     use std::fs::File;
//     use std::io::{BufReader, Read};
//
//     #[test]
//     fn parse_mega() {
//         let mut s = String::new();
//         let data = BufReader::new(File::open(r#"D:\Downloads\freedict-eng-deu-1.9-fd1.src\eng-deu\freedict-P5.dtd"#).unwrap()).read_to_string(&mut s).unwrap();
//         let (x, parsed) = doc_type_no_decl::<_, nom::error::VerboseError<_>>(s.trim()).unwrap();
//         for value in parsed.iter() {
//             println!("{value:?}")
//         }
//     }
// }
//
// // todo:
// // - when a MayBeUnparsed is parsed, try to parse it by resolving the unparsed and reuse the
// //   elements via an interface.