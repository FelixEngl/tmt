pub use documents::*;
pub use logical_structures::*;
pub use physical_structures::*;
pub use errors::*;

mod errors {
    use nom::error::ParseError;
    use crate::topicmodel::dictionary::loader::dtd_parser::AlreadyInUseError;

    pub trait XMLParseError<T>:
    ParseError<T>
    + nom::error::FromExternalError<T, AlreadyInUseError>
    + nom::error::FromExternalError<T, strum::ParseError>
    + nom::error::FromExternalError<T, std::char::CharTryFromError>
    + nom::error::FromExternalError<T, std::num::ParseIntError>
    {}

    impl<S, T> XMLParseError<T> for S
    where S: ParseError<T>
    + nom::error::FromExternalError<T, AlreadyInUseError>
    + nom::error::FromExternalError<T, strum::ParseError>
    + nom::error::FromExternalError<T, std::char::CharTryFromError>
    + nom::error::FromExternalError<T, std::num::ParseIntError>
    {

    }
}


mod documents {
    pub use well_formed_xml_documents::*;
    pub use characters::*;
    pub use character_data_and_markup::*;
    pub use common_syntactic_constructs::*;
    pub use comments::*;
    pub use processing_instruction::*;
    pub use cdata_sections::*;
    pub use prolog_and_xml::*;
    pub use standalone_document_declaration::*;
    pub use white_space_handling::*;
    pub use end_of_line_handling::*;
    pub use language_identification::*;
    pub use cardinality::*;

    mod well_formed_xml_documents {
        use std::fmt::{Display, Formatter};
        use nom::combinator::{map, opt};
        use nom::IResult;
        use nom::multi::many1;
        use nom::sequence::tuple;
        use super::super::{element, misc, prolog, Element, Misc, Prolog, XMLParseError};

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct Document<'a>(pub Prolog<'a>, pub Element<'a>, pub Option<Vec<Misc<'a>>>);

        impl<'a> Display for Document<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}", self.0, self.1)?;
                if let Some(ref misc) = self.2 {
                    for value in misc.iter() {
                        write!(f, "{value}")?;
                    }
                }
                Ok(())
            }
        }

        pub fn document<'a, E: XMLParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Document<'a>, E> {
            map(
                tuple((
                    prolog,
                    element,
                    opt(many1(misc))
                )),
                |(a, b, c)| Document(a, b, c)
            )(s)
        }

    }

    mod characters {
        pub fn is_char(c: char) -> bool {
            matches!(
            c,
            '\u{9}'
            | '\u{A}'
            | '\u{D}'
            | '\u{20}'..='\u{D7FF}'
            | '\u{E000}'..='\u{FFFD}'
            | '\u{10000}'..='\u{10FFFF}'
        )
        }
    }

    // todo: https://www.w3.org/TR/REC-xml/#sec-line-ends

    mod common_syntactic_constructs {
        use nom::character::complete::multispace0;
        use nom::error::ParseError;
        use nom::IResult;
        use nom::sequence::delimited;
        pub use names_and_tokens::*;
        pub use literals::*;
        use super::super::XMLParseError;

        mod names_and_tokens {
            use itertools::Itertools;
            use nom::bytes::complete::{take_while, take_while1};
            use nom::character::complete::char;
            use nom::combinator::{map, recognize};
            use nom::error::{ErrorKind};
            use nom::multi::separated_list1;
            use nom::{AsChar, IResult, InputTakeAtPosition, Parser};
            use std::fmt::{Display, Formatter};
            use std::ops::Deref;
            use nom::sequence::pair;
            use super::super::super::{XMLParseError as ParseError, is_char};

            pub fn dtd_char<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
            where
                T: InputTakeAtPosition,
                <T as InputTakeAtPosition>::Item: AsChar
            {
                input.split_at_position_complete(|value| is_char(value.as_char()))
            }

            pub fn dtd_char1<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
            where
                T: InputTakeAtPosition,
                <T as InputTakeAtPosition>::Item: AsChar
            {
                input.split_at_position1_complete(|value| is_char(value.as_char()), ErrorKind::Char)
            }

            pub fn is_name_start(c: char) -> bool {
                matches!(
                    c,
                    'a'..='z'
                    |'A'..='Z'
                    | '_'
                    | ':'
                    | '\u{C0}'..='\u{D6}'
                    | '\u{D8}'..='\u{F6}'
                    | '\u{F8}'..='\u{2FF}'
                    | '\u{370}'..='\u{37D}'
                    | '\u{37F}'..='\u{1FFF}'
                    | '\u{200C}'..='\u{200D}'
                    | '\u{2070}'..='\u{218F}'
                    | '\u{2C00}'..='\u{2FEF}'
                    | '\u{3001}'..='\u{D7FF}'
                    | '\u{F900}'..='\u{FDCF}'
                    | '\u{FDF0}'..='\u{FFFD}'
                    | '\u{10000}'..='\u{EFFFF}'
                )
            }

            pub fn is_name_char(c: char) -> bool {
                is_name_start(c)
                    || matches!(c, '-' | '.' | '0'..='9' | '\u{B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}')
            }


            #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
            #[repr(transparent)]
            pub struct Name<'a>(&'a str);

            pub fn name<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Name<'a>, E> {
                map(
                    recognize(
                        pair(
                            take_while1(is_name_start),
                            take_while(is_name_char)
                        )
                    ),
                    Name
                )(s)
            }

            impl<'a> Deref for Name<'a> {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }

            impl<'a> Display for Name<'a> {
                delegate::delegate! {
                    to self.0 {
                        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
                    }
                }
            }

            #[derive(Clone, Debug)]
            #[repr(transparent)]
            pub struct Names<'a>(pub Vec<Name<'a>>);

            pub fn names<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Names<'a>, E> {
                map(
                    separated_list1(
                        char('\u{20}'),
                        name
                    ),
                    Names
                )(s)
            }

            impl<'a> Deref for Names<'a> {
                type Target = [Name<'a>];

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl Display for Names<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0.iter().join("\u{20}"))
                }
            }

            #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
            #[repr(transparent)]
            pub struct Nmtoken<'a>(&'a str);

            pub fn nm_token<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Nmtoken<'a>, E> {
                map(
                    take_while1(is_name_char),
                    Nmtoken
                )(s)
            }

            impl<'a> Deref for Nmtoken<'a> {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }

            impl<'a> Display for Nmtoken<'a> {
                delegate::delegate! {
                    to self.0 {
                        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
                    }
                }
            }

            #[derive(Clone, Debug)]
            #[repr(transparent)]
            pub struct Nmtokens<'a>(Vec<Nmtoken<'a>>);

            pub fn nm_tokens<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Nmtokens<'a>, E> {
                map(
                    separated_list1(
                        char('\u{20}'),
                        nm_token
                    ),
                    Nmtokens
                )(s)
            }
            impl<'a> Deref for Nmtokens<'a> {
                type Target = [Nmtoken<'a>];

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl Display for Nmtokens<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0.iter().join("\u{20}"))
                }
            }

        }

        mod literals {
            use super::super::super::physical_structures::{pe_reference, reference, PEReference, Reference};
            use nom::branch::alt;
            use nom::bytes::complete::{take_while, take_while1};
            use nom::character::complete::char;
            use nom::combinator::{map, recognize};
            use super::super::super::XMLParseError as ParseError;
            use nom::multi::many0;
            use nom::sequence::delimited;
            use nom::{IResult, Parser};
            use std::borrow::Cow;
            use std::fmt::{Display, Formatter};
            use std::ops::Deref;
            use itertools::Itertools;
            use strum::Display;

            fn is_pub_id_char(c: char) -> bool {
                matches!(
                    c,
                    '\u{20}'
                    | '\u{D}'
                    | '\u{A}'
                    | 'a'..='z'
                    | 'A'..='Z'
                    | '0'..='9'
                ) || "-'()+,./:=?;!*#@$_%".contains(c)
            }

            fn is_pub_id_char_no_apostroph(c: char) -> bool {
                matches!(
                    c,
                    '\u{20}'
                    | '\u{D}'
                    | '\u{A}'
                    | 'a'..='z'
                    | 'A'..='Z'
                    | '0'..='9'
                ) || "-()+,./:=?;!*#@$_%".contains(c)
            }

            #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Display)]
            pub enum EntityValuePart<'a> {
                #[strum(to_string = "{0}")]
                Raw(&'a str),
                #[strum(to_string = "{0}")]
                PEReference(PEReference<'a>),
                #[strum(to_string = "{0}")]
                Reference(Reference<'a>),
            }

            impl<'a>  EntityValuePart<'a> {
                pub fn as_str(&'a self) -> Cow<'a, str> {
                    match self {
                        EntityValuePart::Raw(value) => {
                            Cow::Borrowed(*value)
                        }
                        EntityValuePart::PEReference(value) => {
                            Cow::Borrowed(value.deref())
                        }
                        EntityValuePart::Reference(value) => {
                            value.as_str()
                        }
                    }
                }
            }



            fn is_raw_entity_value_part(delimiter: char) -> impl Fn(char) -> bool {
                move |c| {
                    c != delimiter && c != '%' && c != '&'
                }
            }

            pub fn entity_value_part<'a, E: ParseError<&'a str>>(delimiter: char) -> impl Parser<&'a str, EntityValuePart<'a>, E> {
                alt((
                    map(recognize(take_while1(is_raw_entity_value_part(delimiter))), EntityValuePart::Raw),
                    map(pe_reference, EntityValuePart::PEReference),
                    map(reference, EntityValuePart::Reference),

                ))
            }

            macro_rules! value_display {
                ($name: ident) => {
                    impl<'a> Display for $name<'a> {
                        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                            if self.0.iter().any(|v| v.as_str().contains('"')) {
                                write!(f, "'{}'", self.0.iter().join(""))
                            } else {
                                write!(f, "\"{}\"", self.0.iter().join(""))
                            }
                        }
                    }
                };
            }

            #[derive(Debug, Clone, Eq, PartialEq, Hash)]
            #[repr(transparent)]
            pub struct EntityValue<'a>(pub Vec<EntityValuePart<'a>>);

            pub fn entity_value<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EntityValue<'a>, E> {
                map(
                    alt((
                        delimited(
                            char('"'),
                            many0(entity_value_part('"')),
                            char('"'),
                        ),
                        delimited(
                            char('\''),
                            many0(entity_value_part('\'')),
                            char('\''),
                        )
                    )),
                    EntityValue
                )(s)
            }

            value_display!(EntityValue);

            #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
            pub enum AttValuePart<'a> {
                #[strum(to_string = "{0}")]
                Raw(&'a str),
                #[strum(to_string = "{0}")]
                Reference(Reference<'a>)
            }

            impl<'a> AttValuePart<'a> {
                pub fn as_str(&'a self) -> Cow<'a, str> {
                    match self {
                        AttValuePart::Raw(value) => {
                            Cow::Borrowed(*value)
                        }
                        AttValuePart::Reference(value) => {
                            value.as_str()
                        }
                    }
                }
            }

            fn is_raw_att_value_part(delimiter: char) -> impl Fn(char) -> bool {
                move |c| {
                    c != delimiter && c != '<' && c != '&'
                }
            }

            pub fn att_value_part<'a, E: ParseError<&'a str>>(delimiter: char) -> impl Parser<&'a str, AttValuePart<'a>, E> {
                alt((
                    map(recognize(take_while1(is_raw_att_value_part(delimiter))), AttValuePart::Raw),
                    map(reference, AttValuePart::Reference),
                ))
            }

            #[derive(Debug, Clone, Eq, PartialEq, Hash)]
            #[repr(transparent)]
            pub struct AttValue<'a>(pub Vec<AttValuePart<'a>>);

            pub fn att_value<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, AttValue<'a>, E> {
                map(
                    alt((
                        delimited(
                            char('"'),
                            many0(att_value_part('"')),
                            char('"'),
                        ),
                        delimited(
                            char('\''),
                            many0(att_value_part('\'')),
                            char('\''),
                        )
                    )),
                    AttValue
                )(s)
            }

            value_display!(AttValue);


            macro_rules! literal_display {
                ($name: ident) => {
                    impl<'a> Display for $name<'a> {
                        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                            if self.0.contains('"') {
                                write!(f, "'{}'", self.0)
                            } else {
                                write!(f, "\"{}\"", self.0)
                            }
                        }
                    }
                };
            }

            #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
            #[repr(transparent)]
            pub struct SystemLiteral<'a>(pub &'a str);

            impl<'a> Deref for SystemLiteral<'a> {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }

            literal_display!(SystemLiteral);


            pub fn system_literal<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, SystemLiteral<'a>, E> {
                map(
                    alt((
                        delimited(
                            char('"'),
                            take_while(|value| value != '"'),
                            char('"'),
                        ),
                        delimited(
                            char('\''),
                            take_while(|value| value != '\''),
                            char('\''),
                        )
                    )),
                    SystemLiteral
                )(s)
            }

            #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
            #[repr(transparent)]
            pub struct PubidLiteral<'a>(pub &'a str);

            impl<'a> Deref for PubidLiteral<'a> {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }

            literal_display!(PubidLiteral);

            pub fn pub_id_literal<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, PubidLiteral<'a>, E> {
                map(
                    alt((
                        delimited(
                            char('"'),
                            take_while(is_pub_id_char),
                            char('"'),
                        ),
                        delimited(
                            char('\''),
                            take_while(is_pub_id_char_no_apostroph),
                            char('\''),
                        )
                    )),
                    PubidLiteral
                )(s)
            }
        }

        // customs
        pub fn eq<'a, E: XMLParseError<&'a str>>(s: &'a str) -> IResult<&'a str, char, E> {
            delimited(
                multispace0,
                nom::character::complete::char('='),
                multispace0
            )(s)
        }
    }

    mod character_data_and_markup {
        use std::fmt::{Display, Formatter};
        use nom::bytes::complete::take_while;
        use nom::combinator::map;
        use super::super::XMLParseError as ParseError;
        use nom::IResult;

        #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct CharData<'a>(pub &'a str);

        impl Display for CharData<'_> {
            delegate::delegate! {
                to self.0 {
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
                }
            }
        }

        pub fn char_data<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, CharData<'a>, E> {
            map(
                nom::combinator::verify(
                    take_while(|value: char| value != '<' && value != '&'),
                    |value: &str| !value.contains("]]>")
                ),
                CharData
            )(s)
        }
    }

    mod comments {
        use std::fmt::{Display, Formatter};
        use super::characters::is_char;
        use nom::branch::alt;
        use nom::bytes::complete::{tag, take_while1};
        use nom::character::complete::char;
        use nom::combinator::{map, recognize};
        use super::super::XMLParseError as ParseError;
        use nom::sequence::{delimited, pair};
        use nom::IResult;
        use nom::multi::many0;

        #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct Comment<'a>(pub &'a str);

        impl Display for Comment<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<!--{}-->", self.0)
            }
        }

        fn is_comment_char(c: char) -> bool {
            c != '-' && is_char(c)
        }

        pub fn comment<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Comment<'a>, E> {
            map(
                delimited(
                    tag("<!--"),
                    recognize(many0(
                        alt((
                            take_while1(is_comment_char),
                            recognize(pair(char('-'), take_while1(is_comment_char)))
                        ))
                    )),
                    tag("-->"),
                ),
                Comment
            )(s)
        }
    }

    mod processing_instruction {
        use std::fmt::{Display, Formatter};
        use super::{Name, name};
        use nom::bytes::complete::take_until1;
        use nom::bytes::complete::tag;
        use nom::character::complete::multispace1;
        use nom::combinator::{map, opt, verify};
        use super::super::XMLParseError as ParseError;
        use nom::sequence::{delimited, pair, preceded};
        use nom::IResult;

        #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct PITarget<'a>(pub Name<'a>);

        impl Display for PITarget<'_> {
            delegate::delegate! {
                to self.0 {
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
                }
            }
        }

        pub fn pi_target<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, PITarget<'a>, E> {
            map(
                verify(
                    name,
                    |value| !value.eq_ignore_ascii_case("xml")
                ),
                PITarget
            )(s)
        }

        #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
        pub struct PI<'a>(
            pub PITarget<'a>,
            pub Option<&'a str>
        );

        impl Display for PI<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<?{}", self.0)?;
                if let Some(s) = self.1 {
                    write!(f, " {s}")?;
                }
                write!(f, "?>")
            }
        }

        pub fn pi<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, PI<'a>, E> {
            map(
                delimited(
                    tag("<?"),
                    pair(
                        pi_target,
                        opt(
                            preceded(
                                multispace1,
                                take_until1("?>")
                            )
                        )
                    ),
                    tag("?>"),
                ),
                |(a, b)| PI(a, b)
            )(s)
        }
    }

    mod cdata_sections {
        use std::fmt::{Display, Formatter};
        use nom::bytes::complete::{tag, take_until1};
        use nom::combinator::map;
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::sequence::delimited;

        #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct CDSect<'a>(pub &'a str);

        impl Display for CDSect<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<!CDATA[{}]]>", self.0)
            }
        }

        pub fn cd_sect<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, CDSect<'a>, E> {
            map(
                delimited(
                    tag("<!CDATA["),
                    take_until1("]]>"),
                    tag("]]>")
                ),
                CDSect
            )(s)
        }
    }

    mod prolog_and_xml {
        pub use prolog::*;
        pub use document_type_definition::*;
        pub use external_subset::*;

        mod prolog {
            use std::fmt::{Display, Formatter};
            use itertools::Itertools;
            use nom::branch::alt;
            use nom::bytes::complete::{tag};
            use nom::character::complete::{char, multispace1};
            use nom::combinator::{map, map_res, opt, recognize, value};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::multi::many0;
            use nom::sequence::{delimited, pair, preceded, tuple};
            use thiserror::Error;
            use super::super::super::physical_structures::{encoding_decl, EncodingDecl};
            use super::{doc_type_decl, DocTypeDecl};
            use super::super::comments::{comment, Comment};
            use super::super::common_syntactic_constructs::eq;
            use super::super::processing_instruction::{pi, PI};
            use super::super::standalone_document_declaration::{sd_decl, SDDecl};

            #[derive(Debug, Clone, Hash, Eq, PartialEq)]
            pub struct Prolog<'a>(
                pub Option<XMLDecl<'a>>,
                pub Vec<Misc<'a>>,
                pub Option<(DocTypeDecl<'a>, Vec<Misc<'a>>)>
            );

            impl Display for Prolog<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    if let Some(ref xml_decl) = self.0 {
                        write!(f, "{xml_decl}")?;
                    }
                    write!(f, "{}", self.1.iter().join(""))?;
                    if let Some(ref doc_type_decl) = self.2 {
                        write!(f, "{}{}", doc_type_decl.0, doc_type_decl.1.iter().join(""))?;
                    }
                    Ok(())
                }
            }

            pub fn prolog<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Prolog<'a>, E> {
                map(
                    tuple((
                        opt(xml_decl),
                        many0(misc),
                        opt(pair(doc_type_decl, many0(misc)))
                    )),
                    |(a, b, c)| Prolog(a, b, c)
                )(s)
            }

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            pub struct XMLDecl<'a> (
                pub VersionInfo<'a>,
                pub Option<EncodingDecl<'a>>,
                pub Option<SDDecl>
            );

            impl Display for XMLDecl<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "<?xml{}", self.0)?;
                    if let Some(enc) = self.1 {
                        write!(f, "{enc}")?;
                    }
                    if let Some(enc) = self.2 {
                        write!(f, "{enc}")?;
                    }
                    write!(f, "?>")
                }
            }

            #[derive(Debug, Error)]
            #[error("The {0} was declared multiple times!")]
            pub struct AlreadyInUseError(&'static str);

            pub fn xml_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, XMLDecl<'a>, E> {
                delimited(
                    tag("<?xml"),
                    map_res(
                        tuple(
                            (
                                version_info,
                                opt(enc_or_sd),
                                opt(enc_or_sd)
                            )
                        ),
                        |(version_info, b, c)| {
                            let encoding_decl = match b {
                                None => {
                                    return Ok(
                                        XMLDecl(version_info, None, None)
                                    )
                                }
                                Some(EncDeclOrSDDecl::Enc(value)) => {
                                    Some(value)
                                }
                                Some(_) => {
                                    match c {
                                        None => {
                                            None
                                        }
                                        Some(EncDeclOrSDDecl::Enc(value)) => {
                                            Some(value)
                                        }
                                        Some(_) => {
                                            return Err(AlreadyInUseError("The standalone was declared multiple times!"));
                                        }
                                    }
                                }
                            };

                            let sd_decl = match b {
                                None => {
                                    return Ok(XMLDecl(version_info, encoding_decl, None))
                                }
                                Some(EncDeclOrSDDecl::SD(value)) => {
                                    Some(value)
                                }
                                Some(_) => {
                                    match c {
                                        None => {
                                            None
                                        }
                                        Some(EncDeclOrSDDecl::SD(value)) => {
                                            Some(value)
                                        }
                                        Some(_) => {
                                            return Err(AlreadyInUseError("The encoding was declared multiple times!"));
                                        }
                                    }
                                }
                            };

                            Ok(XMLDecl(version_info, encoding_decl, sd_decl))
                        }
                    ),
                    tag("?>")
                )(s)
            }

            enum EncDeclOrSDDecl<'a> {
                Enc(EncodingDecl<'a>),
                SD(SDDecl)
            }

            fn enc_or_sd<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EncDeclOrSDDecl<'a>, E> {
                alt((
                    map(encoding_decl, EncDeclOrSDDecl::Enc),
                    map(sd_decl, EncDeclOrSDDecl::SD),
                ))(s)
            }


            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            #[repr(transparent)]
            pub struct VersionInfo<'a>(pub VersionNum<'a>);
            impl Display for VersionInfo<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, " version=\"{}\"", self.0)
                }
            }

            pub fn version_info<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, VersionInfo<'a>, E> {
                map(
                    preceded(
                        delimited(multispace1, tag("version"), eq),
                        alt((
                            delimited(
                                char('"'),
                                version_num,
                                char('"'),
                            ),
                            delimited(
                                char('\''),
                                version_num,
                                char('\''),
                            )
                        ))
                    ),
                    VersionInfo
                )(s)
            }

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            #[repr(transparent)]
            pub struct VersionNum<'a>(pub &'a str);

            impl Display for VersionNum<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }

            pub fn version_num<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, VersionNum<'a>, E> {
                map(
                    recognize(
                        preceded(
                            tag("1."),
                            nom::character::complete::digit1
                        )
                    ),
                    VersionNum
                )(s)
            }

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            pub enum Misc<'a> {
                Comment(Comment<'a>),
                PI(PI<'a>),
                Space
            }

            impl Display for Misc<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    match self {
                        Misc::Comment(value) => {
                            write!(f, "{value}")
                        }
                        Misc::PI(value) => {
                            write!(f, "{value}")
                        }
                        Misc::Space => {
                            write!(f, " ")
                        }
                    }
                }
            }

            pub fn misc<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Misc<'a>, E> {
                alt((
                    map(comment, Misc::Comment),
                    map(pi, Misc::PI),
                    value(Misc::Space, multispace1)
                ))(s)
            }
        }

        mod document_type_definition {
            use std::cell::LazyCell;
            use std::fmt::{Display, Formatter};
            use std::hash::{Hash, Hasher};
            use std::iter::FlatMap;
            use std::sync::Arc;
            use derive_more::From;
            use itertools::Itertools;
            use nom::branch::alt;
            use nom::bytes::complete::tag;
            use nom::character::complete::{char, multispace0, multispace1};
            use nom::combinator::{into, map, opt, value};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::multi::many0;
            use nom::sequence::{delimited, preceded, terminated, tuple};
            use strum::Display;
            use thiserror::Error;
            use crate::topicmodel::dictionary::loader::dtd_parser::{content, content_spec, ContentSpec, InnerChildren, XMLParseError};
            use crate::topicmodel::dictionary::loader::dtd_parser::solving::{DTDResolver, ResolverError};
            use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::MayBeUnresolved;
            use super::super::super::logical_structures::{attlist_decl, AttlistDecl, element_decl, ElementDecl};
            use super::super::super::physical_structures::{
                entity_decl,
                EntityDecl,
                external_id,
                ExternalID,
                notation_decl,
                NotationDecl,
                pe_reference,
                PEReference
            };
            use super::super::{comment, Comment};
            use super::super::{pi, PI};
            use super::super::{Name, name};

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display)]
            pub enum DeclSep<'a> {
                #[strum(to_string=" ")]
                Space,
                #[strum(to_string="{0}")]
                PEReference(PEReference<'a>)
            }


            pub fn decl_sep<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DeclSep<'a>, E> {
                alt((
                    map(pe_reference, DeclSep::PEReference),
                    value(DeclSep::Space, multispace1)
                ))(s)
            }

            #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
            pub enum MarkUpDecl<'a> {
                ElementDecl(ElementDecl<'a>),
                AttlistDecl(AttlistDecl<'a>),
                EntityDecl(EntityDecl<'a>),
                NotationDecl(NotationDecl<'a>),
                PI(PI<'a>),
                Comment(Comment<'a>),
            }

            pub fn mark_up_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, MarkUpDecl<'a>, E> {
                alt((
                    into(element_decl::<E>),
                    into(attlist_decl::<E>),
                    into(entity_decl::<E>),
                    into(notation_decl::<E>),
                    into(pi::<E>),
                    into(comment::<E>),
                ))(s)
            }

            #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
            pub enum IntSubsetPart<'a> {
                #[strum(to_string="{0}")]
                MarkupDecl(MarkUpDecl<'a>),
                #[strum(to_string="{0}")]
                DeclSep(DeclSep<'a>)
            }

            fn int_subset_part<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, IntSubsetPart<'a>, E> {
                alt((
                    into(mark_up_decl::<E>),
                    into(decl_sep::<E>),
                ))(s)
            }

            #[derive(Debug, Clone)]
            pub struct IntSubset<'a>(pub Vec<IntSubsetPart<'a>>, DTDResolver<'a>);

            impl<'a> Hash for IntSubset<'a> {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.0.hash(state)
                }
            }

            impl<'a> Eq for IntSubset<'a> {}
            impl<'a> PartialEq for IntSubset<'a> {
                fn eq(&self, other: &Self) -> bool {
                    self.0.eq(&other.0)
                }
            }


            impl<'a> IntSubset<'a> {
                pub fn iter(&self) -> std::slice::Iter<IntSubsetPart<'a>> {
                    self.0.iter()
                }

                pub fn resolve<E: XMLParseError<&'a str>>(&mut self) -> Result<(), ResolverError<'a, E>> {
                    let targets = self.0.iter().filter_map(
                        |value| {
                            if let IntSubsetPart::MarkupDecl(MarkUpDecl::EntityDecl(value)) = value {
                                Some(value)
                            } else {
                                None
                            }
                        }
                    ).collect_vec();
                    self.1.register_complete_list(targets).unwrap();
                    for vale in self.0.iter_mut() {
                        match vale {
                            IntSubsetPart::MarkupDecl(targs) => {
                                match targs {
                                    MarkUpDecl::ElementDecl(element) => {
                                        let resolved = if let Some(resolved) = element.1.as_mut_resolved() {
                                            resolved
                                        } else {
                                            let unres = element.1.as_unresolved().unwrap();
                                            if let Some(resolved) = self.1.resolve(unres) {
                                                element.1.set_resolved(content_spec::<E>(resolved.as_ref())?.1);
                                            } else {
                                                return Err(ResolverError::FailedToResolve(element.0))
                                            }
                                            element.1.as_mut_resolved().unwrap()
                                        };

                                        match resolved {
                                            ContentSpec::Mixed(value) => {
                                                value.resolve(&mut self.1)?;
                                            }
                                            ContentSpec::Children(value) => {
                                                value.resolve(&mut self.1)?;
                                            }
                                            _ => {}
                                        }
                                    }
                                    MarkUpDecl::AttlistDecl(attribute) => {}
                                    _ => {}
                                }
                            }
                            IntSubsetPart::DeclSep(_) => {}
                        }
                    }
                    Ok(())
                }
            }

            impl Display for IntSubset<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0.iter().join(""))
                }
            }

            pub fn int_subset<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, IntSubset<'a>, E> {
                map(
                    many0(int_subset_part),
                    |value| IntSubset(value, Default::default())
                )(s)
            }




            #[derive(Debug, Clone, Hash, Eq, PartialEq)]
            pub struct DocTypeDecl<'a>(
                pub Name<'a>,
                pub Option<ExternalID<'a>>,
                pub Option<IntSubset<'a>>
            );

            impl<'a> DocTypeDecl<'a> {
                pub fn iter<'b>(&'b self) -> FlatMap<std::option::Iter<'b, IntSubset<'a>>, std::slice::Iter<'b, IntSubsetPart<'a>>, impl FnMut(&'b IntSubset<'a>) -> std::slice::Iter<'b, IntSubsetPart<'a>>> {
                    self.2.iter().flat_map(|value| value.iter())
                }
            }

            impl<'a> Display for DocTypeDecl<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "<!DOCTYPE {}", self.0)?;
                    if let Some(ref ext) = self.1 {
                        write!(f, " {}", ext)?;
                    }
                    if let Some(ref sub) = self.2 {
                        write!(f, " [{}]", sub)?;
                    }
                    write!(f, ">")
                }
            }

            pub fn doc_type_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DocTypeDecl<'a>, E> {
                map(
                    delimited(
                        terminated(tag("<!DOCTYPE"), multispace1),
                        tuple((
                            name,
                            opt(preceded(multispace1, external_id)),
                            preceded(multispace0, opt(delimited(char('['), int_subset, char(']'))))
                        )),
                        char('>')
                    ),
                    |(name, external_id, int_subset)| {
                        DocTypeDecl (name, external_id, int_subset)
                    }
                )(s)
            }

            #[inline(always)]
            pub fn doc_type_no_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, IntSubset<'a>, E> {
                int_subset(s)
            }
        }

        mod external_subset {
            use std::fmt::{Display, Formatter};
            use itertools::Itertools;
            use nom::branch::alt;
            use nom::combinator::{map, opt};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::multi::many0;
            use nom::sequence::pair;
            use strum::Display;
            use super::super::super::logical_structures::{conditional_sect, ConditionalSect};
            use super::super::prolog_and_xml::{decl_sep, mark_up_decl, DeclSep, MarkUpDecl};
            use super::super::super::physical_structures::{text_decl, TextDecl};

            #[derive(Debug, Clone, Hash, Eq, PartialEq)]
            pub struct ExtSubset<'a>(
                pub Option<TextDecl<'a>>,
                pub ExtSubsetDecl<'a>,
            );

            impl<'a> Display for ExtSubset<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    if let Some(ref v) = self.0 {
                        write!(f, "{v}")?;
                    }
                    write!(f, "{}", self.1)
                }
            }

            pub fn ext_subset<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ExtSubset<'a>, E> {
                map(
                    pair(
                        opt(text_decl),
                        ext_subset_decl
                    ),
                    |(a, b)| ExtSubset(a, b)
                )(s)
            }

            #[derive(Debug, Clone, Hash, Eq, PartialEq, Display)]
            pub enum ExtSubsetDeclPart<'a>{
                #[strum(to_string = "{0}")]
                MarkUpDecl(MarkUpDecl<'a>),
                #[strum(to_string = "{0}")]
                ConditionalSect(ConditionalSect<'a>),
                #[strum(to_string = "{0}")]
                DeclSep(DeclSep<'a>)
            }

            pub fn ext_subset_decl_part<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ExtSubsetDeclPart<'a>, E> {
                alt((
                    map(mark_up_decl, ExtSubsetDeclPart::MarkUpDecl),
                    map(conditional_sect, ExtSubsetDeclPart::ConditionalSect),
                    map(decl_sep, ExtSubsetDeclPart::DeclSep),
                ))(s)
            }

            #[derive(Debug, Clone, Hash, Eq, PartialEq)]
            pub struct ExtSubsetDecl<'a>(pub Vec<ExtSubsetDeclPart<'a>>);

            impl<'a> Display for ExtSubsetDecl<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0.iter().join(""))
                }
            }

            pub fn ext_subset_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ExtSubsetDecl<'a>, E> {
                map(
                    many0(ext_subset_decl_part),
                    ExtSubsetDecl
                )(s)
            }
        }
    }

    mod standalone_document_declaration {
        use nom::branch::alt;
        use nom::bytes::complete::{is_not, tag};
        use nom::character::complete::{char, multispace1};
        use nom::combinator::map_res;
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::sequence::{delimited, preceded};
        use strum::{Display, EnumString};
        use super::common_syntactic_constructs::eq;

        #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display, EnumString)]
        pub enum SDDecl {
            #[strum(to_string=" standalone=\"yes\"", serialize="yes")]
            Yes,
            #[strum(to_string=" standalone=\"no\"", serialize="no")]
            No
        }

        pub fn sd_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, SDDecl, E> {
            preceded(
                delimited(multispace1, tag("standalone"), eq),
                alt((
                    delimited(
                        char('"'),
                        map_res(is_not("\""), |value: &str| value.parse()),
                        char('"'),
                    ),
                    delimited(
                        char('\''),
                        map_res(is_not("'"), |value: &str| value.parse()),
                        char('\''),
                    )
                ))
            )(s)
        }
    }

    mod white_space_handling {
        // see https://www.w3.org/TR/REC-xml/#sec-white-space
    }

    mod end_of_line_handling {
        use std::borrow::Cow;
        use itertools::Itertools;
        use memchr::{memchr2_iter};
        use super::super::XMLParseError as ParseError;
        use nom::{AsBytes, Parser};

        /// see: https://www.w3.org/TR/REC-xml/#sec-line-ends
        pub fn normalize_newlines(s: &str) -> Cow<str> {
            let found = memchr2_iter(b'\r', b'\n', s.as_bytes()).collect_vec();
            if found.is_empty() {
                Cow::Borrowed(s)
            } else {
                let mut last_index = None;
                let bytes = s.as_bytes();

                enum Instruction {
                    Replace,
                    Remove,
                }

                let mut instructions = Vec::with_capacity(found.len());
                let mut new_capacity = bytes.len();
                for index in found.into_iter() {
                    if let Some(last) = last_index.replace(index) {
                        if last + 1 == index {
                            new_capacity -= 1;
                            instructions.push((index, Instruction::Remove));
                            continue
                        }
                    }
                    let current = unsafe{*bytes.get_unchecked(index)};
                    if b'\r' == current {
                        instructions.push((index, Instruction::Replace));
                    }
                }
                if instructions.is_empty() {
                    Cow::Borrowed(s)
                } else {
                    let mut new_string_content = Vec::with_capacity(new_capacity);
                    let mut last_idx = 0usize;
                    for (idx, instruction) in instructions.into_iter() {
                        match instruction {
                            Instruction::Replace => {
                                new_string_content.extend_from_slice(&bytes[last_idx..idx]);
                                new_string_content.push(b'\n');
                                last_idx = idx + 1;
                            }
                            Instruction::Remove => {
                                last_idx += 1;
                            }
                        }
                    }
                    Cow::Owned(String::from_utf8(new_string_content).unwrap())
                }
            }
        }

        #[cfg(test)]
        mod test_normalize_newlines {
            use std::borrow::Cow;
            use super::normalize_newlines;

            const TEST_1: &str = "Hello world, this text does not cause an allocation very good!";
            const TEST_2: &str = "Hello world, this text \n does not cause an allocation \n very good!";
            const TEST_3: &str = "Hello \r world, this text \n does not cause an \r allocation \n very good!";
            const TEST_3_EXP: &str = "Hello \n world, this text \n does not cause an \n allocation \n very good!";
            const TEST_4: &str = "Hello \r\n world, this text \n does not cause an \r\n allocation \n very good!";
            const TEST_4_EXP: &str = "Hello \n world, this text \n does not cause an \n allocation \n very good!";
            const TEST_5: &str = "Hello \r\n world, this text \n\n\n\n does not cause an \r\n\r\n\r\n allocation \n very good!";
            const TEST_5_EXP: &str = "Hello \n world, this text \n does not cause an \n allocation \n very good!";

            #[test]
            fn does_not_allocate_1() {
                assert!(matches!(normalize_newlines(TEST_1), Cow::Borrowed(_)))
            }

            #[test]
            fn does_not_allocate_2() {
                assert!(matches!(normalize_newlines(TEST_2), Cow::Borrowed(_)))
            }

            #[test]
            fn does_replace_carriage_returns() {
                let processed = normalize_newlines(TEST_3);
                assert!(matches!(processed, Cow::Owned(_)));
                assert_eq!(TEST_3_EXP, processed.as_ref());
            }

            #[test]
            fn does_replace_carriage_return_and_line_feed() {
                let processed = normalize_newlines(TEST_4);
                assert!(matches!(processed, Cow::Owned(_)));
                assert_eq!(TEST_4_EXP, processed.as_ref());
            }

            #[test]
            fn does_replace_multiple_carriage_returns_and_line_feeds() {
                let processed = normalize_newlines(TEST_5);
                assert!(matches!(processed, Cow::Owned(_)));
                assert_eq!(TEST_5_EXP, processed.as_ref());
            }
        }
    }

    mod language_identification {
        // see https://www.w3.org/TR/REC-xml/#sec-lang-tag
    }

    // custom

    mod cardinality {
        use std::str::FromStr;
        use nom::bytes::complete::take;
        use nom::combinator::map_res;
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use strum::{Display, EnumString};

        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Display, EnumString)]
        pub enum Cardinality {
            #[strum(to_string="?")]
            ZeroOrOne,
            #[strum(to_string="*")]
            ZeroOrMany,
            #[strum(to_string="+")]
            OneOrMany,
        }

        pub fn cardinality<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Cardinality, E> {
            map_res(take(1usize), Cardinality::from_str)(s)
        }
    }
}

mod logical_structures {
    pub use start_tags_end_tags_and_empty_tags::*;
    pub use element_type_declarations::*;
    pub use element_content::*;
    pub use mixed_content::*;
    pub use attribute_list_declarations::*;
    pub use conditional_sections::*;

    use nom::branch::alt;
    use nom::combinator::map;
    use super::XMLParseError as ParseError;
    use nom::IResult;
    use nom::sequence::tuple;
    use strum::Display;

    #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
    pub enum Element<'a> {
        #[strum(to_string = "{0}")]
        EmptyElementTag(EmptyElementTag<'a>),
        #[strum(to_string = "{0}{1}{2}")]
        Element(STag<'a>, Content<'a>, ETag<'a>)
    }

    pub fn element<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Element<'a>, E> {
        alt((
            map(empty_element_tag, Element::EmptyElementTag),
            map(tuple((s_tag, content, e_tag)), |(a, b, c)| Element::Element(a, b, c)),
        ))(s)
    }


    mod start_tags_end_tags_and_empty_tags {
        use std::fmt::{Display, Formatter};
        use derive_more::From;
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::character::complete::{char, multispace0, multispace1};
        use nom::combinator::{into, map, opt};
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::multi::many1;
        use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
        use strum::Display;
        use super::super::physical_structures::{Reference, reference};
        use super::super::documents::{cd_sect, CDSect,char_data, CharData, comment, Comment,att_value, AttValue, Name, name, eq,pi, PI};
        use super::{element, Element};


        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct STag<'a>(pub Name<'a>, pub Option<Vec<Attribute<'a>>>);

        impl<'a> Display for STag<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<{}", self.0)?;
                if let Some(ref value) = self.1 {
                    for v in value.iter() {
                        write!(f, " {}", v)?;
                    }
                }
                write!(f, ">")
            }
        }

        pub fn s_tag<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, STag<'a>, E> {
            map(
                delimited(
                    char('<'),
                    pair(
                        name,
                        opt(many1(preceded(multispace1, attribute)))
                    ),
                    tag(">")
                ),
                |(a, b)| STag(a, b)
            )(s)
        }



        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct Attribute<'a>(pub Name<'a>, pub AttValue<'a>);

        impl<'a> Display for Attribute<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}={}", self.0, self.1)
            }
        }

        pub fn attribute<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Attribute<'a>, E> {
            map(
                separated_pair(
                    name,
                    eq,
                    att_value
                ),
                |(a, b)| Attribute(a, b)
            )(s)
        }


        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct ETag<'a>(pub Name<'a>);

        impl<'a> Display for ETag<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "</{}>", self.0)
            }
        }

        pub fn e_tag<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ETag<'a>, E> {
            map(
                delimited(
                    tag("</"),
                    name,
                    preceded(multispace0, char('>'))
                ),
                ETag
            )(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
        pub enum InnerContent<'a> {
            #[strum(to_string="{0}")]
            Element(Element<'a>),
            #[strum(to_string="{0}")]
            Reference(Reference<'a>),
            #[strum(to_string="{0}")]
            CDSect(CDSect<'a>),
            #[strum(to_string="{0}")]
            PI(PI<'a>),
            #[strum(to_string="{0}")]
            Comment(Comment<'a>)
        }

        fn inner_content<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, InnerContent<'a>, E> {
            alt((
                into(element::<E>),
                into(reference::<E>),
                into(cd_sect::<E>),
                into(pi::<E>),
                into(comment::<E>),
            ))(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct Content<'a>(
            pub Option<CharData<'a>>,
            pub Option<Vec<(InnerContent<'a>, Option<CharData<'a>>)>>
        );

        impl<'a> Display for Content<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                if let Some(ref a) = self.0 {
                    write!(f, "{a}")?;
                }
                if let Some(ref dat) = self.1 {
                    for (ref i, ref v) in dat.iter(){
                        write!(f, "{i}")?;
                        if let Some(v) = v {
                            write!(f, "{v}")?;
                        }
                    }
                }
                Ok(())
            }
        }

        pub fn content<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Content<'a>, E> {
            map(
                tuple((
                    opt(char_data),
                    opt(
                        many1(
                            pair(
                                inner_content,
                                opt(char_data)
                            )
                        )
                    )
                )),
                |(a, b)| Content(a, b)
            )(s)
        }



        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct EmptyElementTag<'a>(
            pub Name<'a>,
            pub Option<Vec<Attribute<'a>>>
        );

        impl<'a> Display for EmptyElementTag<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<{}", self.0)?;
                if let Some(ref value) = self.1 {
                    for v in value.iter() {
                        write!(f, " {}", v)?;
                    }
                }
                write!(f, "/>")
            }
        }

        pub fn empty_element_tag<'a, E:ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EmptyElementTag<'a>, E> {
            map(
                delimited(
                    char('<'),
                    pair(
                        name,
                        opt(many1(preceded(multispace1, attribute)))
                    ),
                    tag("/>")
                ),
                |(a, b)| EmptyElementTag(a, b)
            )(s)
        }
    }

    mod element_type_declarations {
        use std::fmt::{Display, Formatter};
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::character::complete::char;
        use nom::character::complete::{multispace0, multispace1};
        use nom::combinator::{map, value};
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::sequence::{delimited, preceded, separated_pair, terminated};
        use strum::{Display};
        use crate::topicmodel::dictionary::loader::dtd_parser::solving::{DTDResolver, ResolverError};
        use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, MayBeUnresolved};
        use crate::topicmodel::dictionary::loader::dtd_parser::XMLParseError;
        use super::super::documents::{Name, name};
        use super::element_content::{children, Children};
        use super::mixed_content::{mixed, Mixed};

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct ElementDecl<'a>(pub Name<'a>, pub MayBeUnresolved<'a, ContentSpec<'a>>);

        impl<'a> Display for ElementDecl<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<!ELEMENT {} {}>\n", self.0, self.1)
            }
        }

        pub fn element_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ElementDecl<'a>, E> {
            map(
                delimited(
                    preceded(tag("<!ELEMENT"), multispace1),
                    separated_pair(
                        name,
                        multispace1,
                        may_be_unresolved(content_spec)
                    ),
                    terminated(multispace0, char('>'))
                ),
                |(a, b)| ElementDecl(a, b)
            )(s)
        }

        #[derive(Debug, Clone, Hash, PartialEq, Eq, Display)]
        pub enum ContentSpec<'a> {
            #[strum(to_string="EMPTY")]
            Empty,
            #[strum(to_string="ANY")]
            Any,
            #[strum(to_string="{0}")]
            Mixed(Mixed<'a>),
            #[strum(to_string="{0}")]
            Children(Children<'a>),
        }

        impl<'a> ContentSpec<'a> {
            pub fn resolve<E: XMLParseError<&'a str>>(&mut self, resolver: &DTDResolver<'a>) -> Result<(), ResolverError<'a, E>> {
                match self {
                    ContentSpec::Mixed(value) => {
                        value.resolve(resolver)
                    }
                    ContentSpec::Children(value) => {
                        value.resolve(resolver)
                    }
                    _ => Ok(())
                }
            }
        }

        pub fn content_spec<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ContentSpec<'a>, E> {
            alt((
                value(ContentSpec::Empty, tag("EMPTY")),
                value(ContentSpec::Any, tag("ANY")),
                map(mixed, ContentSpec::Mixed),
                map(children, ContentSpec::Children),
            ))(s)
        }


    }

    mod element_content {
        use std::fmt::{Display, Formatter};
        use std::ops::Deref;
        use derive_more::From;
        use itertools::{Itertools};
        use nom::branch::alt;
        use nom::character::complete::{char, multispace0};
        use nom::combinator::{into, map, opt, verify};
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::multi::separated_list1;
        use nom::sequence::{delimited, pair, preceded, terminated};
        use strum::{Display};
        use crate::topicmodel::dictionary::loader::dtd_parser::solving::{DTDResolver, ResolverError};
        use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, MayBeUnresolved};
        use crate::topicmodel::dictionary::loader::dtd_parser::XMLParseError;
        use super::super::documents::{cardinality, Cardinality, Name, name};

        #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
        pub enum InnerChildren<'a> {
            #[strum(to_string = "{0}")]
            Choice(Choice<'a>),
            Seq(Seq<'a>)
        }

        fn inner_child<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, InnerChildren<'a>, E> {
            alt((
                into(choice::<E>),
                into(seq::<E>),
            ))(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct Children<'a>(
            pub InnerChildren<'a>,
            pub Option<Cardinality>
        );

        impl<'a> Children<'a> {

            pub fn resolve<E: XMLParseError<&'a str>>(&mut self, resolver: &DTDResolver<'a>) -> Result<(), ResolverError<'a, E>> {
                match &mut self.0 {
                    InnerChildren::Choice(value) => {
                        value.resolve(resolver)
                    }
                    InnerChildren::Seq(value) => {
                        value.resolve(resolver)
                    }
                }
            }
        }

        impl<'a> Display for Children<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)?;
                if let Some(mu) = self.1 {
                    write!(f, "{}", mu)?;
                }
                Ok(())
            }
        }

        pub fn children<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Children<'a>, E> {
            map(
                pair(
                    inner_child,
                    opt(cardinality)
                ),
                |(a, b)| Children(a, b)
            )(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
        pub enum CPInner<'a> {
            #[strum(to_string = "{0}")]
            Name(Name<'a>),
            #[strum(to_string = "{0}")]
            Choice(Choice<'a>),
            #[strum(to_string = "{0}")]
            Seq(Seq<'a>),
        }

        impl<'a> CPInner<'a> {
            pub fn resolve<E: XMLParseError<&'a str>>(&mut self, resolver: &DTDResolver<'a>) -> Result<(), ResolverError<'a, E>> {
                match self {
                    CPInner::Name(_) => {
                        Ok(())
                    }
                    CPInner::Choice(value) => {
                        value.resolve(resolver)
                    }
                    CPInner::Seq(value) => {
                        value.resolve(resolver)
                    }
                }
            }
        }

        fn cp_inner<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, CPInner<'a>, E> {
            alt((
                into(name::<E>),
                into(choice::<E>),
                into(seq::<E>),
            ))(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct CP<'a>(pub MayBeUnresolved<'a, CPInner<'a>>, pub Option<Cardinality>);

        impl<'a> CP<'a> {
            pub fn resolve<E: XMLParseError<&'a str>>(&mut self, resolver: &DTDResolver<'a>) -> Result<(), ResolverError<'a, E>> {
                if resolver.resolve_if_possible(&mut self.0, cp_inner)? {
                    self.0.as_mut_resolved().unwrap().resolve(resolver)
                } else {
                    use std::borrow::Borrow;
                    let name: &Name<'a> = self.0.as_unresolved().unwrap().borrow();
                    Err(ResolverError::FailedToResolve(name.clone()))
                }
            }
        }

        impl<'a> Display for CP<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)?;
                if let Some(c) = self.1 {
                    write!(f, "{c}")?;
                }
                Ok(())
            }
        }

        pub fn cp<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, CP<'a>, E> {
            map(
                pair(
                    may_be_unresolved(cp_inner),
                    opt(cardinality)
                ),
                |(a, b)| CP(a, b)
            )(s)
        }


        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct Choice<'a>(pub Vec<CP<'a>>);

        impl<'a> Choice<'a> {
            pub fn resolve<E: XMLParseError<&'a str>>(&mut self, resolver: &DTDResolver<'a>) -> Result<(), ResolverError<'a, E>> {
                for value in self.0.iter_mut() {
                    value.resolve(resolver)?;
                }
                Ok(())
            }
        }

        impl<'a> Display for Choice<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "({})", self.0.iter().join("|"))
            }
        }

        pub fn choice<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Choice<'a>, E> {
            map(
                delimited(
                    terminated(char('('), multispace0),
                    verify(
                        separated_list1(
                            delimited(multispace0, char('|'), multispace0),
                            cp
                        ),
                        |value: &Vec<CP<'a>>| { value.len() > 1 }
                    ),
                    preceded(multispace0, char(')'))
                ),
                Choice
            )(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct Seq<'a>(pub Vec<CP<'a>>);

        impl<'a> Seq<'a> {
            pub fn resolve<E: XMLParseError<&'a str>>(&mut self, resolver: &DTDResolver<'a>) -> Result<(), ResolverError<'a, E>> {
                for value in self.0.iter_mut() {
                    value.resolve(resolver)?;
                }
                Ok(())
            }
        }


        impl<'a> Display for Seq<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "({})", self.0.iter().join(","))
            }
        }

        pub fn seq<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Seq<'a>, E> {
            map(
                delimited(
                    terminated(char('('), multispace0),
                    separated_list1(
                        delimited(multispace0, char(','), multispace0),
                        cp
                    ),
                    preceded(multispace0, char(')'))
                ),
                Seq
            )(s)
        }
    }

    mod mixed_content {
        use std::fmt::{Display, Formatter};
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::character::complete::{char, multispace0};
        use nom::combinator::{map, opt, value};
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::multi::{many1, separated_list1};
        use nom::sequence::{delimited, preceded, terminated, tuple};
        use crate::topicmodel::dictionary::loader::dtd_parser::solving::{DTDResolver, ResolverError};
        use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, MayBeUnresolved};
        use crate::topicmodel::dictionary::loader::dtd_parser::{names, XMLParseError};
        use super::super::documents::{Name, name};


        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct Mixed<'a>(pub Option<Vec<MayBeUnresolved<'a, Name<'a>>>>);

        impl<'a> Mixed<'a> {
            pub fn resolve<E: XMLParseError<&'a str>>(&mut self, resolver: &DTDResolver<'a>) -> Result<(), ResolverError<'a, E>> {
                if let Some(names2) = self.0.as_mut() {
                    for targ in names2.iter_mut() {
                        if targ.is_unresolved() {
                            let unres = targ.as_unresolved().unwrap();
                            if let Some(res) = resolver.resolve(unres) {
                                match names::<E>(res.as_ref()) {
                                    Ok((_, mut values)) => {
                                        if let Some(value) = values.0.pop() {
                                            targ.set_resolved(value);
                                        }
                                        names2.extend(values.into_iter().copied().map(MayBeUnresolved::resolved))
                                    }
                                    Err(e) => {
                                        return Err(ResolverError::ParserFailed(e))
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
        }

        impl<'a> Display for Mixed<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "(#PCDATA")?;
                if let Some(ref names) = self.0 {
                    for v in names {
                        write!(f, "|  {v}")?;
                    }
                }
                write!(f, ")")
            }
        }

        pub fn mixed<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Mixed<'a>, E> {
            map(
                alt((
                    delimited(
                        tuple((char('('), multispace0, tag("#PCDATA"))),
                        map(
                            many1(
                                preceded(
                                    delimited(multispace0, char('|'), multispace0),
                                    may_be_unresolved(name)
                                ),
                            ),
                            Some
                        ),
                        preceded(multispace0, tag(")*"))
                    ),
                    value(
                        None,
                        delimited(
                            terminated(char('('), multispace0),
                            tag("#PCDATA"),
                            preceded(multispace0, char(')'))
                        )
                    )
                )),
                Mixed
            )(s)
        }
    }

    mod attribute_list_declarations {
        use std::fmt::{Display, Formatter};
        use std::str::FromStr;
        use derive_more::From;
        use itertools::Itertools;
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::character::complete::{char, multispace0, multispace1};
        use nom::combinator::{into, map, opt};
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::multi::{many1};
        use nom::sequence::{delimited, pair, preceded, terminated, tuple};
        use attribute_types::{AttType, att_type};
        use attribute_defaults::{default_decl, DefaultDecl};
        use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{may_be_unresolved, may_be_unresolved_wrapped, MayBeUnresolved};
        use super::super::documents::{Name, name};

        // todo: https://www.w3.org/TR/REC-xml/#AVNormalize

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct AttlistDecl<'a>(
            pub Name<'a>,
            pub Option<Vec<MayBeUnresolved<'a, AttDef<'a>>>>
        );

        impl<'a> Display for AttlistDecl<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<!ATTLIST {}", self.0)?;
                if let Some(ref att) = self.1 {
                    write!(f, "{}", att.iter().join(""))?;
                }
                write!(f, ">\n")
            }
        }

        pub fn attlist_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, AttlistDecl<'a>, E> {
            map(
                delimited(
                    terminated(tag("<!ATTLIST"), multispace1),
                    pair(name, opt(many1(may_be_unresolved_wrapped(att_def, |value| preceded(multispace1, value))))),
                    preceded(multispace0, char('>'))
                ),
                |(a, b)| AttlistDecl(a, b)
            )(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct AttDef<'a>(
            pub Name<'a>,
            pub MayBeUnresolved<'a, AttType<'a>>,
            pub DefaultDecl<'a>
        );

        impl<'a> Display for AttDef<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, " {} {} {}", self.0, self.1, self.2)
            }
        }

        pub fn att_def<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, AttDef<'a>, E> {
            map(
                tuple((
                    preceded(multispace1, name),
                    preceded(multispace1, may_be_unresolved(att_type)),
                    preceded(multispace1, default_decl),
                )),
                |(a, b, c)| AttDef(a, b, c)
            )(s)
        }

        pub mod attribute_types {
            use std::fmt::{Display, Formatter};
            use std::str::FromStr;
            use derive_more::From;
            use itertools::Itertools;
            use nom::branch::alt;
            use nom::bytes::complete::tag;
            use nom::character::complete::{alpha1, char, multispace0, multispace1};
            use nom::combinator::{into, map, map_res, value};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::multi::separated_list1;
            use nom::sequence::{delimited, preceded, terminated, tuple};
            use strum::{Display, EnumString};
            use super::super::super::documents::{Name, name, Nmtoken, nm_token};

            #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
            pub enum AttType<'a> {
                #[strum(to_string = "CDATA")]
                StringType,
                #[strum(to_string = "{0}")]
                TokenizedType(TokenizedType),
                #[strum(to_string = "{0}")]
                EnumeratedType(EnumeratedType<'a>)
            }

            pub fn att_type<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, AttType<'a>, E> {
                alt((
                    value(AttType::StringType, tag("CDATA")),
                    map(tokenized_type, AttType::TokenizedType),
                    map(enumerated_type, AttType::EnumeratedType),
                ))(s)
            }

            #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display, EnumString)]
            #[strum(serialize_all = "UPPERCASE")]
            pub enum TokenizedType {
                Id,
                IdRef,
                IfRefs,
                Entity,
                Entities,
                NmToken,
                NmTokens
            }

            pub fn tokenized_type<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, TokenizedType, E> {
                map_res(alpha1, TokenizedType::from_str)(s)
            }

            #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
            pub enum EnumeratedType<'a> {
                #[strum(to_string = "{0}")]
                NotationType(NotationType<'a>),
                #[strum(to_string = "{0}")]
                Enumeration(Enumeration<'a>)
            }

            pub fn enumerated_type<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EnumeratedType<'a>, E> {
                alt((
                    into(notation_type::<E>),
                    into(enumeration::<E>),
                ))(s)
            }

            #[derive(Debug, Clone, Eq, PartialEq, Hash)]
            #[repr(transparent)]
            pub struct NotationType<'a>(pub Vec<Name<'a>>);

            impl<'a> Display for NotationType<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "NOTATION ({})", self.0.iter().join("|"))
                }
            }

            pub fn notation_type<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, NotationType<'a>, E> {
                map(
                    delimited(
                        tuple((tag("NOTATION"), multispace1, char('('), multispace0)),
                        separated_list1(
                            delimited(multispace0, char('|'), multispace0),
                            name
                        ),
                        preceded(multispace0, char(')')),
                    ),
                    NotationType
                )(s)
            }

            #[derive(Debug, Clone, Eq, PartialEq, Hash)]
            #[repr(transparent)]
            pub struct Enumeration<'a>(pub Vec<Nmtoken<'a>>);

            impl<'a> Display for Enumeration<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "({})", self.0.iter().join("|"))
                }
            }

            pub fn enumeration<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Enumeration<'a>, E> {
                map(
                    delimited(
                        preceded(char('('), multispace0),
                        separated_list1(
                            delimited(multispace0, char('|'), multispace0),
                            nm_token
                        ),
                        terminated(multispace0, char(')')),
                    ),
                    Enumeration
                )(s)
            }
        }

        pub mod attribute_defaults {
            use nom::branch::alt;
            use nom::bytes::complete::tag;
            use nom::character::complete::multispace1;
            use nom::combinator::{map, opt, value};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::sequence::{preceded, terminated};
            use strum::Display;
            use super::super::super::documents::{att_value, AttValue};

            #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
            pub enum DefaultDecl<'a> {
                #[strum(to_string = "#REQUIRED")]
                Required,
                #[strum(to_string = "#IMPLIED")]
                Implied,
                #[strum(to_string = "#FIXED {0}")]
                AttValue(AttValue<'a>)
            }

            pub fn default_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DefaultDecl<'a>, E> {
                alt((
                    value(DefaultDecl::Required, tag("#REQUIRED")),
                    value(DefaultDecl::Implied, tag("#IMPLIED")),
                    map(
                        preceded(
                            opt(terminated(tag("#FIXED"), multispace1)),
                            att_value
                        ),
                        DefaultDecl::AttValue
                    ),
                ))(s)
            }
        }

        pub mod attribute_value_normalisation {
            // todo: https://www.w3.org/TR/REC-xml/#AVNormalize
        }
    }

    mod conditional_sections {
        use std::fmt::{Display, Formatter};
        use derive_more::From;
        use itertools::Itertools;
        use nom::branch::alt;
        use nom::bytes::complete::{tag, take_until1};
        use nom::character::complete::{char, multispace0};
        use nom::combinator::{into, map, opt};
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::multi::{many0, many1};
        use nom::sequence::{delimited, pair};
        use strum::Display;
        use super::super::documents::{ExtSubsetDecl, ext_subset_decl};

        #[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
        pub struct Ignore<'a>(pub &'a str);

        impl<'a> Display for Ignore<'a> {
            delegate::delegate! {
                to self.0 {
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
                }
            }
        }

        pub fn ignore<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Ignore<'a>, E> {
            map(
                nom::combinator::verify(
                    take_until1("]]>"),
                    |value: &str| !value.contains("<![")
                ),
                Ignore
            )(s)
        }

        #[derive(Debug, Clone, Hash, Eq, PartialEq)]
        pub struct IgnoreSectContents<'a>(
            pub Ignore<'a>,
            pub Option<Vec<(Box<IgnoreSectContents<'a>>, Ignore<'a>)>>
        );

        impl<'a> Display for IgnoreSectContents<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)?;
                if let Some(ref rest) = self.1 {
                    for (cont, ign) in rest.iter(){
                        write!(f, "<![{}]]>{}", cont, ign)?;
                    }
                }
                Ok(())
            }
        }

        pub fn ignore_sect_contents<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, IgnoreSectContents<'a>, E> {
            map(
                pair(
                    ignore,
                    opt(
                        many1(
                            pair(
                                map(
                                    delimited(
                                        tag("<!["),
                                        ignore_sect_contents,
                                        tag("]]>")
                                    ),
                                    Box::new
                                ),
                                ignore
                            )
                        )
                    )
                ),
                |(a, b)| {
                    IgnoreSectContents(a, b)
                }
            )(s)
        }

        #[derive(Debug, Clone, Hash, Eq, PartialEq)]
        pub struct IgnoreSect<'a>(pub Vec<IgnoreSectContents<'a>>);

        impl<'a> Display for IgnoreSect<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<![IGNORE[{}]]>", self.0.iter().join(""))
            }
        }

        pub fn ignore_sect<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, IgnoreSect<'a>, E> {
            map(
                delimited(
                    delimited(
                        tag("<!["),
                        delimited(
                            multispace0,
                            tag("IGNORE"),
                            multispace0
                        ),
                        char('['),

                    ),
                    many0(ignore_sect_contents),
                    tag("]]>")
                ),
                IgnoreSect
            )(s)
        }


        #[derive(Debug, Clone, Hash, Eq, PartialEq)]
        pub struct IncludeSect<'a>(pub ExtSubsetDecl<'a>);

        impl<'a> Display for IncludeSect<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<![INCLUDE[{}]]>", self.0)
            }
        }

        pub fn include_sect<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, IncludeSect<'a>, E> {
            map(
                delimited(
                    delimited(
                        tag("<!["),
                        delimited(
                            multispace0,
                            tag("INCLUDE"),
                            multispace0
                        ),
                        char('['),

                    ),
                    ext_subset_decl,
                    tag("]]>")
                ),
                IncludeSect
            )(s)
        }

        #[derive(Debug, Clone, Hash, Eq, PartialEq, Display, From)]
        pub enum ConditionalSect<'a> {
            #[strum(to_string="{0}")]
            IncludeSect(IncludeSect<'a>),
            #[strum(to_string="{0}")]
            IgnoreSect(IgnoreSect<'a>)
        }

        pub fn conditional_sect<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ConditionalSect<'a>, E> {
            alt((
                into(include_sect::<E>),
                into(ignore_sect::<E>),
            ))(s)
        }
    }
}

mod physical_structures {
    pub use character_and_entity_references::*;
    pub use entity_declarations::*;
    pub use parsed_entities::*;

    mod character_and_entity_references {
        use super::super::documents::{Name, name};
        use itertools::Itertools;
        use nom::branch::alt;
        use nom::bytes::complete::{tag};
        use nom::character::complete::{char, digit1, hex_digit1};
        use nom::combinator::{into, map, map_res};
        use super::super::XMLParseError as ParseError;
        use nom::sequence::delimited;
        use nom::IResult;
        use std::borrow::{Borrow, Cow};
        use std::fmt::{Display, Formatter};
        use std::ops::Deref;
        use derive_more::From;
        use strum::Display;


        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        #[repr(transparent)]
        pub struct CharRef(pub char);

        impl CharRef {
            pub fn as_char(&self) -> char {
                self.0
            }
        }

        impl Deref for CharRef {
            type Target = char;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Display for CharRef {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "&#{};", self.0 as u32)
            }
        }

        pub fn char_ref<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, CharRef, E> {
            map_res(
                alt((
                    delimited(
                        tag("&#"),
                        map_res(digit1, |value| u32::from_str_radix(value, 10)),
                        char(';'),
                    ),
                    delimited(
                        tag("&#x"),
                        map_res(hex_digit1, |value| u32::from_str_radix(value, 16)),
                        char(';'),
                    )
                )),
                |value| {
                    char::try_from(value).map(CharRef)
                }
            )(s)
        }


        #[derive(Debug, Copy, Clone, Eq, Hash, Display, From)]
        pub enum Reference<'a> {
            #[strum(to_string = "{0}")]
            EntityRef(EntityRef<'a>),
            #[strum(to_string = "{0}")]
            CharRef(CharRef),
        }

        impl<'a> Reference<'a> {
            pub fn as_str(&'a self) -> Cow<'a, str> {
                match self {
                    Reference::EntityRef(value) => {
                        Cow::Borrowed(value)
                    }
                    Reference::CharRef(value) => {
                        Cow::Owned(value.to_string())
                    }
                }
            }
        }

        impl<'a> PartialEq for Reference<'a> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::EntityRef(a), Self::EntityRef(b)) => {
                        a == b
                    },
                    (Self::EntityRef(a), Self::CharRef(b)) => {
                        a.chars().exactly_one().is_ok_and(|value| value == b.as_char())
                    },
                    (Self::CharRef(a), Self::EntityRef(b)) => {
                        b.chars().exactly_one().is_ok_and(|value| value == a.as_char())
                    },
                    (Self::CharRef(a), Self::CharRef(b)) => {
                        a == b
                    }
                }
            }
        }

        pub fn reference<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Reference<'a>, E> {
            alt((
                into(char_ref::<E>),
                into(entity_ref::<E>),
            ))(s)
        }

        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct InnerEntityRef<'a>(pub Name<'a>);

        impl<'a> Borrow<Name<'a>> for InnerEntityRef<'a> {
            fn borrow(&self) -> &Name<'a> {
                &self.0
            }
        }

        impl<'a> Deref for InnerEntityRef<'a> {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Display for InnerEntityRef<'_> {
            delegate::delegate! {
                to self.0 {
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
                }
            }
        }

        fn inner_entity_ref<'a, E: ParseError<&'a str>>(c: char) -> impl FnMut(&'a str) -> IResult<&'a str, InnerEntityRef<'a>, E> {
            map(
                delimited(
                    char(c),
                    name,
                    char(';'),
                ),
                InnerEntityRef
            )
        }

        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct EntityRef<'a>(pub InnerEntityRef<'a>);

        impl<'a> Borrow<Name<'a>> for EntityRef<'a> {
            fn borrow(&self) -> &Name<'a> {
                self.0.borrow()
            }
        }

        impl<'a> Deref for EntityRef<'a> {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Display for EntityRef<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "&{};", self.0)
            }
        }

        #[inline(always)]
        pub fn entity_ref<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EntityRef<'a>, E> {
            map(inner_entity_ref('&'), EntityRef)(s)
        }

        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct PEReference<'a>(pub InnerEntityRef<'a>);

        impl<'a> Deref for PEReference<'a> {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<'a> Borrow<Name<'a>> for PEReference<'a> {
            fn borrow(&self) -> &Name<'a> {
                self.0.borrow()
            }
        }

        impl Display for PEReference<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "%{};", self.0)
            }
        }

        #[inline(always)]
        pub fn pe_reference<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, PEReference<'a>, E> {
            map(inner_entity_ref('%'), PEReference)(s)
        }



    }


    mod entity_declarations {
        use std::fmt::{Display, Formatter};
        use derive_more::From;
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::character::complete::{char, multispace0, multispace1};
        use nom::combinator::{into, map, opt};
        use super::super::XMLParseError as ParseError;
        use nom::IResult;
        use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
        use strum::Display;
        use crate::topicmodel::dictionary::loader::dtd_parser::physical_structures::entity_declarations::external_entities::NDataDecl;
        use super::super::documents::{entity_value, EntityValue, Name, name};
        pub use external_entities::*;

        #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
        pub enum EntityDecl<'a> {
            GEDecl(GEDecl<'a>),
            PEDecl(PEDecl<'a>),
        }

        pub fn entity_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EntityDecl<'a>, E> {
            alt((
                into(ge_decl::<E>),
                into(pe_decl::<E>)
            ))(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct GEDecl<'a>(pub Name<'a>, pub EntityDef<'a>);

        impl<'a> Display for GEDecl<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<!ENTITY {} {}>\n", self.0, self.1)
            }
        }

        pub fn ge_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, GEDecl<'a>, E> {
            map(
                delimited(
                    terminated(tag("<!ENTITY"), multispace1),
                    separated_pair(
                        name,
                        multispace1,
                        entity_def
                    ),
                    preceded(multispace0, char('>'))
                ),
                |(a, b)| GEDecl(a, b)
            )(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct PEDecl<'a>(pub Name<'a>, pub PEDef<'a>);

        impl<'a> Display for PEDecl<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "<!ENTITY % {} {}>", self.0, self.1)
            }
        }

        pub fn pe_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, PEDecl<'a>, E> {
            map(
                delimited(
                    tuple((tag("<!ENTITY"), multispace1, char('%'), multispace1)),
                    separated_pair(
                        name,
                        multispace1,
                        pe_def
                    ),
                    preceded(multispace0, char('>'))
                ),
                |(a, b)| PEDecl(a, b)
            )(s)
        }


        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub enum EntityDef<'a> {
            EntityValue(EntityValue<'a>),
            ExternalId(ExternalID<'a>, Option<NDataDecl<'a>>)
        }


        impl<'a> Display for EntityDef<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    EntityDef::EntityValue(value) => {
                        Display::fmt(value, f)
                    }
                    EntityDef::ExternalId(id, data_decl) => {
                        write!(f, "{id}")?;
                        if let Some(data_decl) = data_decl {
                            write!(f, "{data_decl}")?;
                        }
                        Ok(())
                    }
                }
            }
        }

        pub fn entity_def<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EntityDef<'a>, E> {
            alt((
                map(entity_value, EntityDef::EntityValue),
                map(pair(external_id, opt(n_data_decl)), |(a, b)| EntityDef::ExternalId(a, b)),
            ))(s)
        }

        #[derive(Debug, Clone, Eq, PartialEq, Hash, Display, From)]
        pub enum PEDef<'a> {
            #[strum(to_string = "{0}")]
            EntityValue(EntityValue<'a>),
            #[strum(to_string = "{0}")]
            ExternalId(ExternalID<'a>)
        }

        pub fn pe_def<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, PEDef<'a>, E> {
            alt((
                into(entity_value::<E>),
                into(external_id::<E>),
            ))(s)
        }


        // internal_entities (Not needed)

        pub mod external_entities {
            use std::fmt::{Display, Formatter};
            use nom::branch::alt;
            use nom::bytes::complete::tag;
            use nom::character::complete::multispace1;
            use nom::combinator::map;
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::sequence::{delimited, preceded, separated_pair, terminated};
            use crate::topicmodel::dictionary::loader::dtd_parser::{pub_id_literal, system_literal};
            use super::super::super::documents::{PubidLiteral, SystemLiteral, Name, name};

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            pub struct ExternalID<'a>(pub Option<PubidLiteral<'a>>, pub SystemLiteral<'a>);

            impl<'a> ExternalID<'a> {
                pub fn is_public(&self) -> bool {
                    self.0.is_some()
                }
            }

            impl Display for ExternalID<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    if let Some(ref pu) = self.0 {
                        write!(f, "PUBLIC {} {}", pu, self.1)
                    } else {
                        write!(f, "SYSTEM {}", self.1)
                    }
                }
            }

            pub fn external_id<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ExternalID<'a>, E> {
                alt((
                    map(preceded(terminated(tag("SYSTEM"), multispace1), system_literal),
                        |value| ExternalID(None, value)
                    ),
                    map(
                        preceded(terminated(tag("PUBLIC"), multispace1), separated_pair(pub_id_literal, multispace1, system_literal)),
                        |(a, b)| {
                            ExternalID(Some(a), b)
                        }
                    ),
                ))(s)
            }

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            #[repr(transparent)]
            pub struct NDataDecl<'a>(pub Name<'a>);

            impl Display for NDataDecl<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, " NDATA {}", self.0)
                }
            }

            pub fn n_data_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, NDataDecl<'a>, E> {
                preceded(
                    delimited(multispace1, tag("NDATA"), multispace1),
                    map(name, NDataDecl)
                )(s)
            }
        }

    }

    mod parsed_entities {
        pub use text_declarations::*;
        pub use well_formed_parsed_entities::*;
        pub use encoding_declaration::*;
        pub use notation_declaration::*;

        mod text_declarations {
            use std::fmt::{Display, Formatter};
            use nom::bytes::complete::tag;
            use nom::combinator::{map, opt};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::sequence::{delimited, pair};
            use super::{encoding_decl, EncodingDecl};
            use super::super::super::documents::{version_info, VersionInfo};

            #[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
            pub struct TextDecl<'a>(pub EncodingDecl<'a>, pub Option<VersionInfo<'a>>);

            impl<'a> Display for TextDecl<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "<?xml")?;
                    if let Some(ref v) = self.1 {
                        write!(f, "{v}")?;
                    }
                    write!(f, "{}?>", self.0)
                }
            }

            pub fn text_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, TextDecl<'a>, E> {
                map(
                    delimited(
                        tag("<?xml"),
                        pair(
                            opt(version_info),
                            encoding_decl
                        ),
                        tag("?>")
                    ),
                    |(a,b)| {
                        TextDecl(b, a)
                    }
                )(s)
            }
        }

        mod well_formed_parsed_entities {
            use std::fmt::{Display, Formatter};
            use nom::combinator::{map, opt};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::sequence::pair;
            use super::super::super::logical_structures::{content, Content};
            use super::{text_decl, TextDecl};

            #[derive(Debug, Clone, Eq, PartialEq, Hash)]
            pub struct ExtParsedEnt<'a>(pub Option<TextDecl<'a>>, pub Content<'a>);

            impl<'a> Display for ExtParsedEnt<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    if let Some(ref decl) = self.0 {
                        write!(f, "{decl}")?;
                    }
                    write!(f, "{}", self.1)
                }
            }

            pub fn ext_parsed_ent<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, ExtParsedEnt<'a>, E> {
                map(
                    pair(
                        opt(text_decl),
                        content
                    ),
                    |(a, b)| ExtParsedEnt(a, b)
                )(s)
            }
        }

        mod encoding_declaration {
            use std::fmt::{Display, Formatter};
            use nom::branch::alt;
            use nom::bytes::complete::{tag, take_while};
            use nom::character::complete::{alpha1, char, multispace1};
            use nom::combinator::{map, recognize};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::sequence::{delimited, pair, preceded};
            use super::super::super::documents::eq;

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            #[repr(transparent)]
            pub struct EncodingDecl<'a>(pub EncName<'a>);

            impl Display for EncodingDecl<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, " encoding=\"{}\"", self.0)
                }
            }

            pub fn encoding_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EncodingDecl<'a>, E> {
                map(
                    preceded(
                        delimited(multispace1, tag("encoding"), eq),
                        alt((
                            delimited(
                                char('"'),
                                enc_name,
                                char('"'),
                            ),
                            delimited(
                                char('\''),
                                enc_name,
                                char('\''),
                            )
                        ))
                    ),
                    EncodingDecl
                )(s)
            }

            #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
            #[repr(transparent)]
            pub struct EncName<'a>(pub &'a str);

            impl Display for EncName<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }

            pub fn enc_name<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, EncName<'a>, E> {
                map(
                    recognize(
                        pair(
                            alpha1,
                            take_while(|value| {
                                nom::AsChar::is_alpha(value) || value == '.' || value == '_' || value == '-'
                            })
                        )
                    ),
                    EncName
                )(s)
            }
        }

        mod notation_declaration {
            use std::fmt::{Display, Formatter};
            use derive_more::From;
            use nom::branch::alt;
            use nom::bytes::complete::tag;
            use nom::character::complete::{char, multispace0, multispace1};
            use nom::combinator::{into, map};
            use super::super::super::XMLParseError as ParseError;
            use nom::IResult;
            use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
            use strum::Display;
            use super::super::super::documents::{pub_id_literal, Name, PubidLiteral, name};
            use super::super::super::physical_structures::{external_id, ExternalID};

            #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Display, From)]
            pub enum InnerNotationDeclId<'a> {
                ExternalId(ExternalID<'a>),
                PublicId(PublicID<'a>),
            }

            pub fn inner_notation_decl_id<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, InnerNotationDeclId<'a>, E> {
                alt((
                    into(external_id::<E>),
                    into(public_id::<E>)
                ))(s)
            }

            #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
            pub struct NotationDecl<'a>(pub Name<'a>, pub InnerNotationDeclId<'a>);

            impl<'a> Display for NotationDecl<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "<!NOTATION {} {}>\n", self.0, self.1)
                }
            }

            pub fn notation_decl<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, NotationDecl<'a>, E> {
                map(
                    delimited(
                        terminated(tag("<!NOTATION"), multispace1),
                        separated_pair(
                            name,
                            multispace1,
                            inner_notation_decl_id
                        ),
                        preceded(multispace0, char('>'))
                    ),
                    |(a, b)| NotationDecl(a, b)
                )(s)
            }

            #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
            #[repr(transparent)]
            pub struct PublicID<'a>(pub PubidLiteral<'a>);

            impl<'a> Display for PublicID<'a> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "PUBLIC {}", self.0)
                }
            }

            pub fn public_id<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, PublicID<'a>, E> {
                map(
                    preceded(pair(tag("PUBLIC"), multispace1), pub_id_literal),
                    PublicID
                )(s)
            }
        }
    }
}

pub mod solving {
    // todo: https://www.w3.org/TR/REC-xml/#intern-replacement

    use std::borrow::{Borrow, Cow};
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;
    use std::fmt::{Debug, Display};
    use std::sync::{Arc, Mutex};
    use itertools::Itertools;
    use nom::{IResult, Parser};
    use thiserror::Error;
    use crate::topicmodel::dictionary::loader::dtd_parser::{DeclSep, EntityDecl, EntityDef, EntityValue, EntityValuePart, IntSubset, IntSubsetPart, MarkUpDecl, Name, PEDef, Reference, XMLParseError};
    use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::{MayBeUnresolved, UnresolvedReference};

    #[derive(Debug, Default, Clone)]
    pub struct DTDResolver<'a> {
        resolved: Arc<Mutex<HashMap<Name<'a>, Cow<'a, str>>>>,
    }

    impl<'a> DTDResolver<'a> {

        pub fn resolve_if_possible<T, E, F>(&self, target: &mut MayBeUnresolved<'a, T>, mut parser: F) -> Result<bool, ResolverError<'a, E>>
        where
            E: XMLParseError<&'a str>,
            F: Parser<&'a str, T, E>,
            T: Display + Debug + 'a + Clone
        {
            if target.is_unresolved(){
                let name: &Name<'a> = target.as_unresolved().unwrap().borrow();
                if let Some(resolved) = self.resolve(name) {
                    match parser.parse(resolved.as_ref()) {
                        Ok((_, value)) => {
                            target.set_resolved(value);
                        }
                        Err(err) => {
                            return Err(ResolverError::ParserFailed(err))
                        }
                    }
                } else {
                    return Ok(false)
                }
            }
            Ok(true)
        }

        pub fn register_complete(&mut self, values: &IntSubset<'a>) -> Result<(), usize> {
            let targets = values.iter().filter_map(
                |value| {
                    if let IntSubsetPart::MarkupDecl(MarkUpDecl::EntityDecl(value)) = value {
                        Some(value)
                    } else {
                        None
                    }
                }
            ).collect_vec();
            self.register_complete_list(targets)
        }

        pub fn register_complete_list(&mut self, targets: Vec<&EntityDecl<'a>>) -> Result<(), usize> {
            let mut ct = 0usize;
            let mut last = None;
            loop {
                for value in targets.iter().copied() {
                    if self.register(value) {
                        ct+=1;
                    }
                }
                if let Some(last) = last.replace(ct) {
                    if ct == last {
                        break Err(targets.len() - ct)
                    }
                }
                if ct == targets.len() {
                    break Ok(())
                }
            }
        }

        pub fn register(&mut self, entity_decl: &EntityDecl<'a>) -> bool {
            let mut resolver = self.resolved.lock().unwrap();
            match entity_decl {
                EntityDecl::GEDecl(decl) => {
                    if resolver.contains_key(&decl.0) {
                        return true;
                    }
                    match &decl.1 {
                        EntityDef::EntityValue(value) => {
                            match self.resolve_entity_def(value) {
                                None => {
                                    false
                                }
                                Some(value) => {
                                    resolver.insert(decl.0, value);
                                    true
                                }
                            }
                        }
                        EntityDef::ExternalId(_, _) => {
                            // todo: https://www.w3.org/TR/REC-xml/#NT-ExternalID
                            log::warn!("ExternalID not supported!");
                            false
                        }
                    }
                }
                EntityDecl::PEDecl(decl) => {
                    if self.resolved.lock().unwrap().contains_key(&decl.0) {
                        return true;
                    }
                    match &decl.1 {
                        PEDef::EntityValue(value) => {
                            match self.resolve_entity_def(value) {
                                None => {
                                    false
                                }
                                Some(value) => {
                                    self.resolved.lock().unwrap().insert(decl.0, value);
                                    true
                                }
                            }
                        }
                        PEDef::ExternalId(_) => {
                            // todo: https://www.w3.org/TR/REC-xml/#NT-ExternalID
                            log::warn!("ExternalID not supported!");
                            false
                        }
                    }
                }
            }
        }

        pub fn resolve<Q: Borrow<Name<'a>>>(&self, a: &Q) -> Option<&Cow<'a, str>> {
            let name = a.borrow();
            self.resolved.lock().unwrap().get(name)
        }

        fn resolve_entity_def(&self, entity_value: &EntityValue<'a>) -> Option<Cow<'a, str>> {
            match entity_value.0.len(){
                0 => Some(Cow::Borrowed("")),
                1 => {
                    self.resolve_entity_def_value_part(unsafe{entity_value.0.get_unchecked(0)})
                }
                _ => {
                    let mut new_value = String::new();
                    for value in entity_value.0.iter() {
                        new_value.push_str(self.resolve_entity_def_value_part(value)?.as_ref());
                    }
                    Some(Cow::Owned(new_value))
                }
            }
        }

        fn resolve_entity_def_value_part(&self, value_part: &EntityValuePart<'a>) -> Option<Cow<'a, str>> {
            match value_part {
                EntityValuePart::Raw(value) => {
                    Some(Cow::Borrowed(*value))
                }
                EntityValuePart::PEReference(value) => {
                    Some(self.resolved.lock().unwrap().get(&value.0.0)?.clone())
                }
                EntityValuePart::Reference(value) => {
                    match value {
                        Reference::EntityRef(value) => {
                            Some(self.resolved.lock().unwrap().get(&value.0.0)?.clone())
                        }
                        Reference::CharRef(value) => {
                            Some(Cow::Owned(value.as_char().to_string()))
                        }
                    }
                }
            }
        }
    }

    #[derive(Debug, Error)]
    pub enum ResolverError<'a, E: XMLParseError<&'a str>> {
        #[error("Failed to register {0} elements!")]
        FailedToRegisterCompletely(usize),
        #[error("Failed to resolve {0}")]
        FailedToResolve(Name<'a>),
        #[error(transparent)]
        ParserFailed(#[from] nom::Err<E>)
    }


}

/// Because we are a primitive parser, we resolve everything after the fact
pub mod unresolved_helper {
    use std::borrow::Borrow;
    use std::fmt::{Debug, Display, Formatter};
    use std::marker::PhantomData;
    use std::ops::Deref;
    use derive_more::From;
    use itertools::Either;
    use nom::branch::alt;
    use nom::combinator::{into, map};
    use nom::{IResult, Parser};
    use strum::Display;
    use crate::topicmodel::dictionary::loader::dtd_parser::{entity_ref, pe_reference, EntityRef, InnerEntityRef, Name, PEReference};
    use crate::topicmodel::dictionary::loader::dtd_parser::errors::XMLParseError;

    #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
    pub enum MayBeUnresolvedRepr<'a, T> where T: Display + Debug + Clone + 'a {
        #[strum(to_string = "{0}")]
        Resolved(T),
        #[strum(to_string = "{0}")]
        Unresolved(UnresolvedReference<'a>)
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct MayBeUnresolved<'a, T> where T: Display + Debug + Clone + 'a {
        inner: MayBeUnresolvedRepr<'a, T>
    }

    impl<'a, T> MayBeUnresolved<'a, T> where T: Display + Debug + Clone + 'a {
        pub fn is_unresolved(&self) -> bool {
            matches!(self.inner, MayBeUnresolvedRepr::Unresolved(_))
        }

        pub fn as_resolved(&self) -> Option<&T> {
            match self.inner {
                MayBeUnresolvedRepr::Resolved(ref value) => {
                    Some(value)
                }
                MayBeUnresolvedRepr::Unresolved(_) => {
                    None
                }
            }
        }

        pub fn as_mut_resolved(&mut self) -> Option<&mut T> {
            match &mut self.inner {
                MayBeUnresolvedRepr::Resolved(value) => {
                    Some(value)
                }
                MayBeUnresolvedRepr::Unresolved(_) => {
                    None
                }
            }
        }

        pub fn as_unresolved(&self) -> Option<&UnresolvedReference<'a>> {
            match self.inner {
                MayBeUnresolvedRepr::Resolved(_) => {
                    None
                }
                MayBeUnresolvedRepr::Unresolved(ref value) => {
                    Some(value)
                }
            }
        }

        pub fn set_resolved(&mut self, resolved: T) -> MayBeUnresolved<'a, T> {
            MayBeUnresolved {
                inner: std::mem::replace(&mut self.inner, MayBeUnresolvedRepr::Resolved(resolved))
            }
        }

        pub fn unresolved(reference: UnresolvedReference<'a>) -> Self{
            Self {
                inner: MayBeUnresolvedRepr::Unresolved(reference)
            }
        }

        pub fn resolved(value: T) -> Self{
            Self {
                inner: MayBeUnresolvedRepr::Resolved(value)
            }
        }
    }

    impl<'a, T> AsRef<MayBeUnresolvedRepr<'a, T>> for MayBeUnresolved<'a, T> where T: Display + Debug + Clone + 'a {
        fn as_ref(&self) -> &MayBeUnresolvedRepr<'a, T> {
            &self.inner
        }
    }

    impl<'a, T> AsMut<MayBeUnresolvedRepr<'a, T>> for MayBeUnresolved<'a, T> where T: Display + Debug + Clone + 'a {
        fn as_mut(&mut self) -> &mut MayBeUnresolvedRepr<'a, T> {
            &mut self.inner
        }
    }

    impl<'a, T> Display for MayBeUnresolved<'a, T> where T: Display + Debug + Clone + 'a {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            Display::fmt(&self.inner, f)
        }
    }

    impl<'a, T> MayBeUnresolved<'a, T> where T: Display + Debug + Clone + 'a {

    }

    pub struct UnresolvedReferenceFn<'a, E> {
        _dat: PhantomData<fn(&'a ()) -> E>
    }

    impl<'a, E> UnresolvedReferenceFn<'a, E> {
        pub fn new() -> Self {
            Self { _dat: PhantomData }
        }
    }

    impl<'a, E> Parser<&'a str, UnresolvedReference<'a>, E> for UnresolvedReferenceFn<'a, E>
        where
            E: XMLParseError<&'a str>
    {
        fn parse(&mut self, input: &'a str) -> IResult<&'a str, UnresolvedReference<'a>, E> {
            unresolved_reference(input)
        }
    }

    /// Helps the parser to parse an element or attlist with somne unresolved data.
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display, From)]
    pub enum UnresolvedReference<'a> {
        #[strum(to_string="{0}")]
        EntityRef(EntityRef<'a>),
        #[strum(to_string="{0}")]
        PEReference(PEReference<'a>)
    }

    impl<'a> Borrow<Name<'a>> for UnresolvedReference<'a> {
        fn borrow(&self) -> &Name<'a> {
            self.deref()
        }
    }

    impl<'a> Deref for UnresolvedReference<'a> {
        type Target = Name<'a>;

        fn deref(&self) -> &Self::Target {
            match self {
                UnresolvedReference::EntityRef(a) => {a.borrow()}
                UnresolvedReference::PEReference(a) => {a.borrow()}
            }
        }
    }

    pub fn unresolved_reference<'a, E: XMLParseError<&'a str>>(s: &'a str) -> IResult<&'a str, UnresolvedReference<'a>, E> {
        alt((
            into(entity_ref::<E>),
            into(pe_reference::<E>),
        ))(s)
    }

    pub fn may_be_unresolved<'a, O1, E: XMLParseError<&'a str>, F>(parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, MayBeUnresolved<'a, O1>, E>
        where
            F: Parser<&'a str, O1, E>,
            O1: Display + Debug + Clone + 'a
    {
        alt((
            map(parser, MayBeUnresolved::resolved),
            map(unresolved_reference, MayBeUnresolved::unresolved),
        ))
    }

    pub fn may_be_unresolved_wrapped<'a, O1, E: XMLParseError<&'a str>, F, W, Q>(parser: F, wrapper: W) -> impl FnMut(&'a str) -> IResult<&'a str, MayBeUnresolved<'a, O1>, E>
    where
        F: Parser<&'a str, O1, E>,
        O1: Display + Debug + Clone + 'a,
        W: Fn(UnresolvedReferenceFn<'a, E>) -> Q,
        Q: Parser<&'a str, UnresolvedReference<'a>, E>
    {
        alt((
            map(parser, MayBeUnresolved::resolved),
            map(wrapper(UnresolvedReferenceFn::new()), MayBeUnresolved::unresolved),
        ))
    }
}


#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{BufReader, Read};
    use nom::{Finish, IResult};
    use crate::topicmodel::dictionary::loader::dtd_parser::{att_def, att_value, attlist_decl, doc_type_no_decl, element_decl, mixed};
    use crate::topicmodel::dictionary::loader::dtd_parser::solving::DTDResolver;
    use crate::topicmodel::dictionary::loader::dtd_parser::unresolved_helper::may_be_unresolved;

    #[test]
    fn can_parse(){
        const EXAMPLE_DOC: &str = r#"<!DOCTYPE TVSCHEDULE [
                <!ELEMENT TVSCHEDULE (CHANNEL+)>
                <!ELEMENT CHANNEL (BANNER,DAY+)>
                <!ELEMENT BANNER (#PCDATA)>
                <!ELEMENT DAY (DATE,(HOLIDAY|PROGRAMSLOT+)+)>
                <!ELEMENT HOLIDAY (#PCDATA)>
                <!ELEMENT DATE (#PCDATA)>
                <!ELEMENT PROGRAMSLOT (TIME,TITLE,DESCRIPTION?)>
                <!ELEMENT TIME (#PCDATA)>
                <!ELEMENT TITLE (#PCDATA)>
                <!ELEMENT DESCRIPTION (#PCDATA)>

                <!ATTLIST TVSCHEDULE NAME CDATA #REQUIRED>
                <!ATTLIST CHANNEL CHAN CDATA #REQUIRED>
                <!ATTLIST PROGRAMSLOT VTR CDATA #IMPLIED>
                <!ATTLIST TITLE RATING CDATA #IMPLIED>
                <!ATTLIST TITLE LANGUAGE CDATA #IMPLIED>
            ]>"#;

//         println!("{:?}", doc_type_no_decl::<nom::error::VerboseError<_>>(r#"<!ENTITY % att.global.linking.attributes '
// corresp CDATA  #IMPLIED'>
// <!ENTITY % att.global.linking.attribute.corresp '
// corresp CDATA  #IMPLIED'>
//
// <!--end of predeclared classes -->
// <!ENTITY % att.ascribed.attributes '
// who CDATA  #IMPLIED'>"#));


 //        let decl = seq::<nom::error::VerboseError<_>>(
 //            r#"(teiHeader,((_DUMMY_model.resourceLike+,text?) |
 // text))"#
 //        );

        // println!("{:?}", attlist_decl()::<nom::error::VerboseError<_>>("(_DUMMY_model.resourceLike+,text?)"));
        // println!("{:?}", cp::<nom::error::VerboseError<_>>("_DUMMY_model.resourceLike+"));
        // println!("{:?}", cp::<nom::error::VerboseError<_>>("text?)"));

 //        match element_decl::<nom::error::VerboseError<_>>(r#"<!ELEMENT bibl ( #PCDATA |
 // _DUMMY_model.gLike |
 // %model.highlighted; |
 // %model.pPart.data; |
 // %model.pPart.edit; |
 // _DUMMY_model.segLike |
 // %model.ptrLike; |
 // %model.biblPart; |
 // %model.global;)*>"#).finish() {
 //            Ok(a) => {
 //                println!("Success: {a:?}")
 //            }
 //            Err(b) => {
 //                println!("{b}")
 //            }
 //        }

        match mixed::<nom::error::VerboseError<_>>(r#"( #PCDATA |
 _DUMMY_model.gLike |
 %model.highlighted; |
 %model.pPart.data; |
 %model.pPart.edit; |
 _DUMMY_model.segLike |
 %model.ptrLike; |
 %model.biblPart; |
 %model.global;)*>"#).finish() {
            Ok(a) => {
                println!("Success: {a:?}")
            }
            Err(b) => {
                println!("{b}")
            }
        }
        match mixed::<nom::error::VerboseError<_>>(r#"( #PCDATA )>"#).finish() {
            Ok(a) => {
                println!("Success: {a:?}")
            }
            Err(b) => {
                println!("{b}")
            }
        }


        // match nom::Parser::parse(&mut may_be_unresolved(att_def::<nom::error::VerboseError<_>>), r#"%att.global.attribute.xmlid;"#).finish() {
        //     Ok(a) => {
        //         println!("Success: {a:?}")
        //     }
        //     Err(b) => {
        //         println!("{b}")
        //     }
        // }
        // match att_value::<nom::error::VerboseError<_>>(r#""http://www.tei-c.org/ns/1.0">"#).finish() {
        //     Ok(a) => {
        //         println!("Success: {a:?}")
        //     }
        //     Err(b) => {
        //         println!("{b}")
        //     }
        // }


        // let mut x = entity_value::<nom::error::VerboseError<_>>;
        // use nom::Parser;
        //
        // match x.parse(r#"'%data.name;'"#) {
        //     Ok(value) => {
        //         println!("{value:?}")
        //     }
        //     Err(value) => {
        //         println!("{value}")
        //     }
        // }

        // let parsed = doc_type_decl::<nom::error::VerboseError<_>>(EXAMPLE_DOC).unwrap();
        //
        //
        //
        // println!("{:?}", parsed.1)
    }

    #[test]
    fn parse_mega() {
        let mut s = String::new();

        let data = BufReader::new(File::open(r#"D:\Downloads\freedict-eng-deu-1.9-fd1.src\eng-deu\freedict-P5.dtd"#).unwrap()).read_to_string(&mut s).unwrap();
        let (x, parsed) = doc_type_no_decl::<nom::error::VerboseError<_>>(s.trim()).unwrap();
        for value in parsed.iter() {
            println!("{value:?}")
        }

        let mut resolver = DTDResolver::default();
        resolver.register_complete(&parsed).unwrap();
        println!("{resolver:?}")
    }
}

