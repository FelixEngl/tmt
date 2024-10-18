use itertools::Itertools;
use nom::branch::alt;
use nom::error::ParseError;
use nom::Parser;
use strum::EnumString;

enum DTDElement {
    Element,
    Attlist,
    Notation,
    Entity
}


struct ElementSpec {
    name: String,

}

#[derive(Debug, Copy, Clone, EnumString)]
pub enum Multiplicity {
    #[strum(to_string = "?")]
    ZeroOrOne,
    #[strum(to_string = "*")]
    ZeroOrMore,
    #[strum(to_string = "+")]
    OneOrMore
}

enum Content {
    Empty,
    Any,
    Mixed,

}


mod names_and_tokens {
    use itertools::Itertools;
    use nom::bytes::complete::{take_while, take_while1};
    use nom::character::complete::char;
    use nom::combinator::{map, recognize};
    use nom::error::{ErrorKind, ParseError};
    use nom::multi::separated_list1;
    use nom::{AsChar, IResult, InputTakeAtPosition, Parser};
    use std::fmt::{Display, Formatter};
    use std::ops::Deref;

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

    pub fn dtd_char<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
    where
        T: InputTakeAtPosition,
        <T as InputTakeAtPosition>::Item: AsChar
    {
        input.split_at_position_complete(is_char)
    }

    pub fn dtd_char1<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
    where
        T: InputTakeAtPosition,
        <T as InputTakeAtPosition>::Item: AsChar
    {
        input.split_at_position1_complete(is_char, ErrorKind::Char)
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
    pub struct Name<'a>(pub &'a str);

    impl<'a, E: ParseError<&'a str>> Parser<&'a str, Name<'a>, E> for Name<'a> {
        fn parse(&mut self, input: &'a str) -> IResult<&'a str, Name<'a>, E> {
            map(
                recognize(
                    (
                        take_while1(is_name_start),
                        take_while(is_name_char)
                    )
                ),
                Name
            )(input)
        }
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

    impl<'a, E: ParseError<&'a str>> Parser<&'a str, Names<'a>, E> for Names<'a> {
        fn parse(&mut self, input: &'a str) -> IResult<&'a str, Names<'a>, E> {
            map(
                separated_list1(
                    char('\u{20}'),
                    Name
                ),
                Names
            )(input)
        }
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

    impl<'a, E: ParseError<&'a str>> Parser<&'a str, Nmtoken<'a>, E> for Nmtoken<'a> {
        fn parse(&mut self, input: &'a str) -> IResult<&'a str, Nmtoken<'a>, E> {
            map(
                take_while1(is_name_char),
                Nmtoken
            )(input)
        }
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
    pub struct Nmtokens<'a>(pub Vec<Nmtoken<'a>>);

    impl<'a, E: ParseError<&'a str>> Parser<&'a str, Nmtokens<'a>, E> for Nmtokens<'a> {
        fn parse(&mut self, input: &'a str) -> IResult<&'a str, Nmtokens<'a>, E> {
            map(
                separated_list1(
                    char('\u{20}'),
                    Name
                ),
                Nmtokens
            )(input)
        }
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
    use crate::topicmodel::dictionary::loader::dtd_parser::references::{pe_reference, reference, PEReference, Reference};
    use nom::branch::alt;
    use nom::bytes::complete::take_while;
    use nom::character::complete::char;
    use nom::combinator::{map, not, recognize};
    use nom::error::ParseError;
    use nom::multi::many0;
    use nom::sequence::delimited;
    use nom::{IResult, Parser};
    use std::borrow::Cow;
    use std::fmt::{Display, Formatter};
    use std::ops::Deref;
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
            | o if "-'()+,./:=?;!*#@$_%".contains(o))
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
            | o if "-()+,./:=?;!*#@$_%".contains(o))
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Display)]
    pub enum EntityValuePart<'a> {
        #[strum(to_string = "{0}")]
        Raw(&'a str),
        #[strum(to_string = "{0}")]
        PEReference(PEReference<'a>),
        #[strum(to_string = "{0}")]
        Reference(Reference<'a>)
    }

    impl<'a>  EntityValuePart<'a> {
        pub fn as_str(&self) -> Cow<'a, str> {
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
            map(recognize(take_while(is_raw_entity_value_part(delimiter))), EntityValuePart::Raw),
            map(pe_reference, EntityValuePart::PEReference),
            map(reference, EntityValuePart::Reference),
        ))
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct EntityValue<'a>(Vec<EntityValuePart<'a>>);

    pub fn entity_value<E: ParseError<&str>>(s: &str) -> IResult<&str, EntityValue, E> {
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

    impl Display for EntityValue<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.0.iter().any(|v| v.as_str().contains('"')) {
                write!(f, "'{}'", self.0.iter().map(|value| value.to_string()))
            } else {
                write!(f, "\"{}\"", self.0.iter().map(|value| value.to_string()))
            }
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Display)]
    pub enum AttValuePart<'a> {
        #[strum(to_string = "{0}")]
        Raw(&'a str),
        #[strum(to_string = "{0}")]
        Reference(Reference<'a>)
    }

    impl<'a> AttValuePart<'a> {
        pub fn as_str(&self) -> Cow<'a, str> {
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
            map(recognize(take_while(is_raw_att_value_part(delimiter))), AttValuePart::Raw),
            map(reference, AttValuePart::Reference),
        ))
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct AttValue<'a>(Vec<AttValuePart<'a>>);

    pub fn att_value<E: ParseError<&str>>(s: &str) -> IResult<&str, AttValue, E> {
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

    impl Display for AttValue<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.0.iter().any(|v| v.as_str().contains('"')) {
                write!(f, "'{}'", self.0.iter().map(|value| value.to_string()))
            } else {
                write!(f, "\"{}\"", self.0.iter().map(|value| value.to_string()))
            }
        }
    }



    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct SystemLiteral<'a>(&'a str);

    impl<'a> Deref for SystemLiteral<'a> {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    impl Display for SystemLiteral<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.0.contains('"') {
                write!(f, "'{}'", self.0)
            } else {
                write!(f, "\"{}\"", self.0)
            }
        }
    }

    pub fn system_literal<E: ParseError<&str>>(s: &str) -> IResult<&str, SystemLiteral, E> {
        map(
            alt((
                delimited(
                    char('"'),
                    take_while(not(char('"'))),
                    char('"'),
                ),
                delimited(
                    char('\''),
                    take_while(not(char('\''))),
                    char('\''),
                )
            )),
            SystemLiteral
        )(s)
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct PubidLiteral<'a>(&'a str);

    impl<'a> Deref for PubidLiteral<'a> {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    impl Display for PubidLiteral<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.0.contains('"') {
                write!(f, "'{}'", self.0)
            } else {
                write!(f, "\"{}\"", self.0)
            }
        }
    }

    pub fn pub_id_literal<E: ParseError<&str>>(s: &str) -> IResult<&str, PubidLiteral, E> {
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

mod character_data {
    use std::fmt::{Display, Formatter};
    use nom::bytes::complete::take_while;
    use nom::combinator::map;
    use nom::error::ParseError;
    use nom::IResult;

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct CharData<'a>(&'a str);

    impl Display for CharData<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub fn char_data<E: ParseError<&str>>(s: &str) -> IResult<&str, CharData, E> {
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
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::is_char;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_while1};
    use nom::character::complete::char;
    use nom::combinator::{map, recognize};
    use nom::error::ParseError;
    use nom::sequence::delimited;
    use nom::IResult;

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct Comment<'a>(&'a str);

    impl Display for Comment<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<!--{}-->", self.0)
        }
    }

    fn is_comment_char(c: char) -> bool {
        c != '-' && is_char(c)
    }

    pub fn comment<E: ParseError<&str>>(s: &str) -> IResult<&str, Comment, E> {
        map(
            delimited(
                tag("<!--"),
                recognize(alt((
                    take_while1(is_comment_char),
                    (char('-'), take_while1(is_comment_char))
                ))),
                tag("-->"),
            ),
            Comment
        )(s)
    }
}

mod processing_instruction {
    use std::fmt::{Display, Formatter};
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::Name;
    use nom::bytes::complete::take_until1;
    use nom::bytes::streaming::tag;
    use nom::character::complete::multispace1;
    use nom::combinator::{map, opt, verify};
    use nom::error::ParseError;
    use nom::sequence::{delimited, pair, preceded};
    use nom::IResult;

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct PITarget<'a>(Name<'a>);

    impl Display for PITarget<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub fn pi_target<E: ParseError<&str>>(s: &str) -> IResult<&str, PITarget, E> {
        map(
            verify(
                Name,
                |value| !value.eq_ignore_ascii_case("xml")
            ),
            PITarget
        )(s)
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    pub struct PI<'a>(
        PITarget<'a>,
        Option<&'a str>
    );

    impl Display for PI<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<?{}", self.0)?;
            if let Some(f) = self.1 {
                write!(f, " {f}")?;
            }
            write!(f, "?>")
        }
    }

    pub fn pi<E: ParseError<&str>>(s: &str) -> IResult<&str, PI, E> {
        delimited(
            tag("<?"),
            map(
                pair(
                    pi_target,
                    opt(
                        preceded(
                            multispace1,
                            take_until1("?>")
                        )
                    )
                ),
                |(a, b)| PI(a, b)
            ),
            tag("?>")
        )
    }
}

mod cdata {
    use std::fmt::{Display, Formatter};
    use nom::bytes::complete::{tag, take_until1};
    use nom::combinator::map;
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::delimited;

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct CDSect<'a>(&'a str);

    impl Display for CDSect<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<!CDATA[{}]]>", self.0)
        }
    }

    pub fn cd_sect<E: ParseError<&str>>(s: &str) -> IResult<&str, CDSect, E> {
        map(
            delimited(
                tag("<!CDATA["),
                take_until1("]]>"),
                tag("]]>")
            ),
            CDSect
        )
    }
}

mod encoding_declaration {
    use std::fmt::{Display, Formatter};
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_while};
    use nom::character::complete::{alpha1, char, multispace0, multispace1};
    use nom::character::is_alphanumeric;
    use nom::combinator::{map, recognize};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::{delimited, pair, preceded};
    use crate::topicmodel::dictionary::loader::dtd_parser::prolog_and_xml::{eq, version_num, VersionInfo};

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct EncodingDecl<'a>(EncName<'a>);

    impl Display for EncodingDecl<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, " encoding=\"{}\"", self.0)
        }
    }

    pub fn encoding_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, EncodingDecl, E> {
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
    pub struct EncName<'a>(&'a str);

    impl Display for EncName<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub fn enc_name<E: ParseError<&str>>(s: &str) -> IResult<&str, EncName, E> {
        map(
            recognize(
                pair(
                    alpha1,
                    take_while(|value| is_alphanumeric(value) || value == b'.' || value == b'_' || value == b'-')
                )
            ),
            EncName
        )
    }
}

mod standalone_declaration {
    use nom::branch::alt;
    use nom::bytes::complete::{is_not, tag};
    use nom::character::complete::{char, multispace0, multispace1};
    use nom::combinator::map_res;
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::{delimited, preceded};
    use strum::{Display, EnumString};
    use crate::topicmodel::dictionary::loader::dtd_parser::prolog_and_xml::eq;

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display, EnumString)]
    pub enum SDDecl {
        #[strum(to_string=" standalone=\"yes\"", serialize="yes")]
        Yes,
        #[strum(to_string=" standalone=\"no\"", serialize="no")]
        No
    }

    pub fn sd_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, SDDecl, E> {
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


mod prolog_and_xml {
    use std::fmt::{Display, Formatter};
    use std::ops::Deref;
    use itertools::Itertools;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_while};
    use nom::character::complete::{char as char2, multispace0, multispace1, space1};
    use nom::combinator::{map, map_res, opt, recognize, value};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::{delimited, preceded, tuple};
    use crate::topicmodel::dictionary::loader::dtd_parser::comments::{comment, Comment};
    use crate::topicmodel::dictionary::loader::dtd_parser::document_type_definition::DocTypeDecl;
    use crate::topicmodel::dictionary::loader::dtd_parser::encoding_declaration::{encoding_decl, EncodingDecl};
    use crate::topicmodel::dictionary::loader::dtd_parser::processing_instruction::{pi, PI};
    use crate::topicmodel::dictionary::loader::dtd_parser::standalone_declaration::{sd_decl, SDDecl};

    pub fn eq<E: ParseError<&str>>(s: &str) -> IResult<&str, char, E> {
        delimited(
            multispace0,
            char2('='),
            multispace0
        )(s)
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct Prolog<'a> {
        decl: XMLDecl<'a>,
        misc: Vec<Misc<'a>>,
        doc_type_decl: Option<(DocTypeDecl<'a>, Vec<Misc<'a>>)>
    }

    impl Display for Prolog<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}{}", self.decl, self.misc.iter().join(""))?;
            if let Some(ref doc_type_decl) = self.doc_type_decl {
                write!(f, "{}{}", doc_type_decl.0, doc_type_decl.1.iter().join(""))?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    pub struct XMLDecl<'a> {
        version_info: VersionInfo<'a>,
        encoding_decl: Option<EncodingDecl<'a>>,
        sd_decl: Option<SDDecl>
    }

    impl Display for XMLDecl<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<?xml{}", self.version_info)?;
            if let Some(enc) = self.encoding_decl {
                write!(f, "{enc}")?;
            }
            if let Some(enc) = self.sd_decl {
                write!(f, "{enc}")?;
            }
            write!(f, "?>")
        }
    }

    enum EncDeclOrSDDecl<'a> {
        Enc(EncodingDecl<'a>),
        SD(SDDecl)
    }

    fn enc_or_sd<E: ParseError<&str>>(s: &str) -> IResult<&str, EncDeclOrSDDecl, E> {
        alt((
            map(encoding_decl, EncDeclOrSDDecl::Enc),
            map(sd_decl, EncDeclOrSDDecl::SD),
        ))
    }

    pub fn xml_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, XMLDecl, E> {
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
                                XMLDecl {
                                    version_info,
                                    encoding_decl: None,
                                    sd_decl: None
                                }
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
                                    return Err("The standalone was declared multiple times!");
                                }
                            }
                        }
                    };

                    let sd_decl = match b {
                        None => {
                            return Ok(
                                XMLDecl {
                                    version_info,
                                    encoding_decl: None,
                                    sd_decl: None
                                }
                            )
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
                                    return Err("The encoding was declared multiple times!");
                                }
                            }
                        }
                    };

                    Ok(
                        XMLDecl {
                            version_info,
                            encoding_decl,
                            sd_decl
                        }
                    )
                }
            ),
            tag("?>")
        )
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct VersionInfo<'a>(VersionNum<'a>);
    impl Display for VersionInfo<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, " version=\"{}\"", self.0)
        }
    }

    pub fn version_info<E: ParseError<&str>>(s: &str) -> IResult<&str, VersionInfo, E> {
        map(
            preceded(
                delimited(multispace1, tag("version"), eq),
                alt((
                    delimited(
                        char2('"'),
                        version_num,
                        char2('"'),
                    ),
                    delimited(
                        char2('\''),
                        version_num,
                        char2('\''),
                    )
                ))
            ),
            VersionInfo
        )(s)
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct VersionNum<'a>(&'a str);

    impl Display for VersionNum<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub fn version_num<E: ParseError<&str>>(s: &str) -> IResult<&str, VersionNum, E> {
        map(
            recognize(
                preceded(
                    tag("1."),
                    nom::character::complete::digit1
                )
            ),
            VersionNum
        )
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

    pub fn misc<E: ParseError<&str>>(s: &str) -> IResult<&str, Misc, E> {
        alt((
            map(comment, Misc::Comment),
            map(pi, Misc::PI),
            value(Misc::Space, multispace1)
        ))
    }
}


mod external_entity_declaration {
    use std::fmt::{Display, Formatter};
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::multispace1;
    use nom::combinator::map;
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::{delimited, preceded, separated_pair, terminated};
    use crate::topicmodel::dictionary::loader::dtd_parser::literals::{PubidLiteral, SystemLiteral};
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::Name;

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    pub struct ExternalID<'a>(SystemLiteral<'a>, Option<PubidLiteral<'a>>);

    impl<'a> ExternalID<'a> {
        pub fn is_public(&self) -> bool {
            self.1.is_some()
        }
    }

    impl Display for ExternalID<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if let Some(ref pu) = self.1 {
                write!(f, "PUBLIC {} {}", pu, self.0)
            } else {
                write!(f, "SYSTEM {}", self.0)
            }
        }
    }

    pub fn external_id<E: ParseError<&str>>(s: &str) -> IResult<&str, ExternalID, E> {
        alt((
            map(preceded(terminated(tag("SYSTEM"), multispace1), SystemLiteral),
            |value| ExternalID(value, None)
            ),
            map(
                preceded(terminated(tag("PUBLIC"), multispace1), separated_pair(PubidLiteral, multispace1, SystemLiteral)),
                |(a, b)| {
                    ExternalID(b, Some(a))
                }
            ),
        ))(s)
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct NDataDecl<'a>(Name<'a>);

    impl Display for NDataDecl<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, " NDATA {}", self.0)
        }
    }

    pub fn n_data_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, NDataDecl, E> {
        preceded(
            delimited(multispace1, tag("NDATA"), multispace1),
            map(Name, NDataDecl)
        )
    }
}

mod document_type_definition {
    use std::fmt::{Display, Formatter};
    use itertools::Itertools;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{char, multispace0, multispace1};
    use nom::combinator::{map, opt, value};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::multi::many0;
    use nom::sequence::{delimited, preceded, terminated, tuple};
    use strum::Display;
    use crate::topicmodel::dictionary::loader::dtd_parser::comments::Comment;
    use crate::topicmodel::dictionary::loader::dtd_parser::external_entity_declaration::{external_id, ExternalID};
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::Name;
    use crate::topicmodel::dictionary::loader::dtd_parser::processing_instruction::PI;
    use crate::topicmodel::dictionary::loader::dtd_parser::references::{pe_reference, PEReference};

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display)]
    pub enum DeclSep<'a> {
        #[strum(to_string=" ")]
        Space,
        #[strum(to_string="{0}")]
        PEReference(PEReference<'a>)
    }


    pub fn decl_sep<E: ParseError<&str>>(s: &str) -> IResult<&str, DeclSep, E> {
        alt((
            map(pe_reference, DeclSep::PEReference),
            value(DeclSep::Space, multispace1)
        ))(s)
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display)]
    pub enum MarkUpDecl<'a> {
        Comment(Comment<'a>),
        PI(PI<'a>),
    }

    pub fn mark_up_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, MarkUpDecl, E> {
        alt((

        ))
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display)]
    pub enum IntSubsetPart<'a> {
        #[strum(to_string="{0}")]
        MarkupDecl(MarkUpDecl<'a>),
        #[strum(to_string="{0}")]
        DeclSep(DeclSep<'a>)
    }

    fn int_subset_part<E: ParseError<&str>>(s: &str) -> IResult<&str, IntSubsetPart, E> {
        alt((
            map(mark_up_decl, IntSubsetPart::MarkupDecl),
            map(decl_sep, IntSubsetPart::DeclSep),
        ))(s)
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct IntSubset<'a>(Vec<IntSubsetPart<'a>>);

    impl Display for IntSubset<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0.iter().join(""))
        }
    }

    pub fn int_subset<E: ParseError<&str>>(s: &str) -> IResult<&str, IntSubset, E> {
        map(
            many0(int_subset_part),
            IntSubset
        )
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct DocTypeDecl<'a> {
        name: Name<'a>,
        external_id: Option<ExternalID<'a>>,
        int_subset: Option<IntSubset<'a>>
    }

    impl<'a> Display for DocTypeDecl<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<!DOCTYPE {}", self.name)?;
            if let Some(ref ext) = self.external_id {
                write!(f, " {}", ext)?;
            }
            if let Some(ref sub) = self.int_subset {
                write!(f, " [{}]", sub)?;
            }
            write!(f, ">")
        }
    }

    pub fn doc_type_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, DocTypeDecl, E> {
        map(
            delimited(
                terminated(tag("<!DOCTYPE"), multispace1),
                tuple((
                    Name,
                    opt(preceded(multispace1, external_id)),
                    preceded(multispace0, opt(delimited(char('['), int_subset, char(']'))))
                )),
                char('>')
            ),
            |(name, external_id, int_subset)| {
                DocTypeDecl {
                    name,
                    external_id,
                    int_subset
                }
            }
        )(s)
    }
}

mod external_subset {
    use std::fmt::{Display, Formatter};
    use itertools::Itertools;
    use nom::branch::alt;
    use nom::combinator::{map, opt};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::multi::many0;
    use nom::sequence::pair;
    use strum::Display;
    use crate::topicmodel::dictionary::loader::dtd_parser::conditional_sections::{conditional_sect, ConditionalSect};
    use crate::topicmodel::dictionary::loader::dtd_parser::document_type_definition::{decl_sep, mark_up_decl, DeclSep, MarkUpDecl};
    use crate::topicmodel::dictionary::loader::dtd_parser::text_declarations::{text_decl, TextDecl};

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct ExtSubset<'a>(
        ExtSubsetDecl<'a>,
        Option<TextDecl<'a>>
    );

    impl<'a> Display for ExtSubset<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if let Some(ref v) = self.1 {
                write!(f, "{v}")?;
            }
            write!(f, "{}", self.0)
        }
    }

    pub fn ext_subset<E: ParseError<&str>>(s: &str) -> IResult<&str, ExtSubset, E> {
        map(
            pair(
                opt(text_decl),
                ext_subset_decl
            ),
            |(a, b)| ExtSubset(b, a)
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

    pub fn ext_subset_decl_part<E: ParseError<&str>>(s: &str) -> IResult<&str, ExtSubsetDeclPart, E> {
        alt((
            map(mark_up_decl, ExtSubsetDeclPart::MarkUpDecl),
            map(conditional_sect, ExtSubsetDeclPart::ConditionalSect),
            map(decl_sep, ExtSubsetDeclPart::DeclSep),
        ))(s)
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct ExtSubsetDecl<'a>(Vec<ExtSubsetDeclPart<'a>>);

    impl<'a> Display for ExtSubsetDecl<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0.iter().join(""))
        }
    }

    pub fn ext_subset_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, ExtSubsetDecl, E> {
        map(
            many0(ext_subset_decl_part),
            ExtSubsetDecl
        )(s)
    }
}

mod text_declarations {
    use std::fmt::{Display, Formatter};
    use nom::bytes::complete::tag;
    use nom::combinator::{map, opt};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::{delimited, pair};
    use crate::topicmodel::dictionary::loader::dtd_parser::encoding_declaration::{encoding_decl, EncodingDecl};
    use crate::topicmodel::dictionary::loader::dtd_parser::prolog_and_xml::{version_info, VersionInfo};

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
    pub struct TextDecl<'a>(EncodingDecl<'a>, Option<VersionInfo<'a>>);

    impl<'a> Display for TextDecl<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<?xml")?;
            if let Some(ref v) = self.1 {
                write!(f, "{v}")?;
            }
            write!(f, "{}?>", self.0)
        }
    }

    pub fn text_decl<E: ParseError<&str>>(s: &str) -> IResult<&str, TextDecl, E> {
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
        )
    }
}

mod conditional_sections {
    use std::fmt::{Display, Formatter};
    use itertools::Itertools;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_until1};
    use nom::character::complete::{char, multispace0};
    use nom::combinator::{map, opt};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::multi::{many0, many1};
    use nom::sequence::{delimited, pair};
    use strum::Display;
    use crate::topicmodel::dictionary::loader::dtd_parser::external_subset::{ext_subset_decl, ExtSubsetDecl};

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
    pub struct Ignore<'a>(&'a str);

    impl<'a> Display for Ignore<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub fn ignore<E: ParseError<&str>>(s: &str) -> IResult<&str, Ignore, E> {
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
        Ignore<'a>,
        Option<Vec<(Box<IgnoreSectContents<'a>>, Ignore<'a>)>>
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

    pub fn ignore_sect_contents<E: ParseError<&str>>(s: &str) -> IResult<&str, IgnoreSectContents, E> {
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
    pub struct IgnoreSect<'a>(Vec<IgnoreSectContents<'a>>);

    impl<'a> Display for IgnoreSect<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<![IGNORE[{}]]>", self.0.iter().join(""))
        }
    }

    pub fn ignore_sect<E: ParseError<&str>>(s: &str) -> IResult<&str, IgnoreSect, E> {
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
    pub struct IncludeSect<'a>(ExtSubsetDecl<'a>);

    impl<'a> Display for IncludeSect<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<![INCLUDE[{}]]>", self.0)
        }
    }

    pub fn include_sect<E: ParseError<&str>>(s: &str) -> IResult<&str, IncludeSect, E> {
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

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Display)]
    pub enum ConditionalSect<'a> {
        #[strum(to_string="{0}")]
        IncludeSect(IncludeSect<'a>),
        #[strum(to_string="{0}")]
        IgnoreSect(IgnoreSect<'a>)
    }

    pub fn conditional_sect<E: ParseError<&str>>(s: &str) -> IResult<&str, ConditionalSect, E> {
        alt((
            map(include_sect, ConditionalSect::IncludeSect),
            map(ignore_sect, ConditionalSect::IgnoreSect),
        ))
    }
}

mod references {
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::Name;
    use itertools::Itertools;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_while1};
    use nom::character::complete::char;
    use nom::combinator::{map, map_res};
    use nom::error::ParseError;
    use nom::sequence::delimited;
    use nom::IResult;
    use std::borrow::{Borrow, Cow};
    use std::fmt::{Display, Formatter};
    use std::ops::Deref;

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    #[repr(transparent)]
    pub struct CharRef(char);

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

    pub fn char_ref<E: ParseError<&str>>(s: &str) -> IResult<&str, CharRef, E> {
        map_res(
            alt((
                delimited(
                    tag("&#"),
                    map_res(take_while1(nom::character::is_digit), |value: &str| u32::from_str_radix(value, 10)),
                    char(';'),
                ),
                delimited(
                    tag("&#x"),
                    map_res(take_while1(nom::character::is_hex_digit), |value: &str| u32::from_str_radix(value, 16)),
                    char(';'),
                )
            )),
            |value| {
                char::try_from(value).map(CharRef)
            }
        )(s)
    }


    #[derive(Debug, Copy, Clone, Eq, Hash)]
    pub enum Reference<'a> {
        EntityRef(EntityRef<'a>),
        CharRef(CharRef),
    }

    impl<'a> Reference<'a> {
        pub fn as_str(&self) -> Cow<'a, str> {
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

    pub fn reference<E: ParseError<&str>>(s: &str) -> IResult<&str, Reference, E> {
        alt((
            map(char_ref, Reference::CharRef),
            map(entity_ref, Reference::EntityRef),
        ))
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct InnerEntityRef<'a>(Name<'a>);

    impl<'a> Deref for InnerEntityRef<'a> {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for InnerEntityRef<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "&{};", self.0)
        }
    }

    fn inner_entity_ref<E: ParseError<&str>>(s: &str) -> IResult<&str, InnerEntityRef, E> {
        map(
            delimited(
                char('&'),
                Name,
                char(';'),
            ),
            InnerEntityRef
        )(s)
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct EntityRef<'a>(InnerEntityRef<'a>);

    impl<'a> Borrow<InnerEntityRef<'a>> for EntityRef<'a> {
        fn borrow(&self) -> &InnerEntityRef<'a> {
            &self.0
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
    pub fn entity_ref<E: ParseError<&str>>(s: &str) -> IResult<&str, EntityRef, E> {
        map(inner_entity_ref, EntityRef)(s)
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct PEReference<'a>(InnerEntityRef<'a>);

    impl<'a> Borrow<InnerEntityRef<'a>> for PEReference<'a> {
        fn borrow(&self) -> &InnerEntityRef<'a> {
            &self.0
        }
    }

    impl<'a> Deref for PEReference<'a> {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for PEReference<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "&{};", self.0)
        }
    }

    #[inline(always)]
    pub fn pe_reference<E: ParseError<&str>>(s: &str) -> IResult<&str, PEReference, E> {
        map(inner_entity_ref, PEReference)(s)
    }
}


mod structure {
    use nom::branch::alt;
    use nom::combinator::map;
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::tuple;
    use strum::Display;
    use crate::topicmodel::dictionary::loader::dtd_parser::tags::{content, e_tag, empty_element_tag, s_tag, Content, ETag, EmptyElementTag, STag};

    #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
    pub enum Element<'a> {
        #[strum(to_string = "{0}")]
        EmptyElementTag(EmptyElementTag<'a>),
        #[strum(to_string = "{0}{1}{2}")]
        Element(STag<'a>, Content<'a>, ETag<'a>)
    }

    pub fn element<E: ParseError<&str>>(s: &str) -> IResult<&str, Element, E> {
        alt((
            map(empty_element_tag, Element::EmptyElementTag),
            map(tuple((s_tag, content, e_tag)), |(a, b, c)| Element::Element(a, b, c)),
        ))
    }

}

mod tags {
    use std::fmt::{Display, Formatter};
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{char, multispace0, multispace1};
    use nom::combinator::{map, opt};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::multi::many1;
    use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
    use strum::Display;
    use crate::topicmodel::dictionary::loader::dtd_parser::cdata::{cd_sect, CDSect};
    use crate::topicmodel::dictionary::loader::dtd_parser::character_data::{char_data, CharData};
    use crate::topicmodel::dictionary::loader::dtd_parser::comments::{comment, Comment};
    use crate::topicmodel::dictionary::loader::dtd_parser::literals::{att_value, AttValue};
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::Name;
    use crate::topicmodel::dictionary::loader::dtd_parser::processing_instruction::{pi, PI};
    use crate::topicmodel::dictionary::loader::dtd_parser::prolog_and_xml::eq;
    use crate::topicmodel::dictionary::loader::dtd_parser::references::{reference, Reference};
    use crate::topicmodel::dictionary::loader::dtd_parser::structure::{element, Element};

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct EmptyElementTag<'a>(
        Name<'a>,
        Option<Vec<Attribute<'a>>>
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

    pub fn empty_element_tag<E:ParseError<&str>>(s: &str) -> IResult<&str, EmptyElementTag, E> {
        map(
            delimited(
                char('<'),
                pair(
                    Name,
                    opt(many1(preceded(multispace1, attribute)))
                ),
                tag("/>")
            ),
            |(a, b)| EmptyElementTag(a, b)
        )(s)
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct STag<'a>(Name<'a>, Option<Vec<Attribute<'a>>>);

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

    pub fn s_tag<E:ParseError<&str>>(s: &str) -> IResult<&str, STag, E> {
        map(
            delimited(
                char('<'),
                pair(
                    Name,
                    opt(many1(preceded(multispace1, attribute)))
                ),
                tag(">")
            ),
            |(a, b)| STag(a, b)
        )(s)
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct ETag<'a>(Name<'a>);

    impl<'a> Display for ETag<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "</{}>", self.0)
        }
    }

    pub fn e_tag<E:ParseError<&str>>(s: &str) -> IResult<&str, ETag, E> {
        map(
            delimited(
                tag("</"),
                Name,
                preceded(multispace0, char('>'))
            ),
            ETag
        )(s)
    }


    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct Attribute<'a>(Name<'a>, AttValue<'a>);

    impl<'a> Display for Attribute<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}={}", self.0, self.1)
        }
    }

    pub fn attribute<E:ParseError<&str>>(s: &str) -> IResult<&str, Attribute, E> {
        map(
            separated_pair(
                Name,
                eq,
                att_value
            ),
            |(a, b)| Attribute(a, b)
        )(s)
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
    pub enum InnerContent<'a> {
        #[strum(to_string="{0}")]
        Element(Box<Element<'a>>),
        #[strum(to_string="{0}")]
        Reference(Reference<'a>),
        #[strum(to_string="{0}")]
        CDSect(CDSect<'a>),
        #[strum(to_string="{0}")]
        PI(PI<'a>),
        #[strum(to_string="{0}")]
        Comment(Comment<'a>)
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct Content<'a>(
        Option<CharData<'a>>,
        Option<Vec<(InnerContent<'a>, Option<CharData<'a>>)>>
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

    pub fn content<E:ParseError<&str>>(s: &str) -> IResult<&str, Content, E> {
        map(
            tuple((
                opt(char_data),
                opt(
                    many1(
                        pair(
                            alt((
                                map(element, |elem| InnerContent::Element(Box::new(elem))),
                                map(reference, InnerContent::Reference),
                                map(cd_sect, InnerContent::CDSect),
                                map(pi, InnerContent::PI),
                                map(comment, InnerContent::Comment),
                            )),
                            opt(char_data)
                        )
                    )
                )
            )),
            |(a, b)| Content(a, b)
        )
    }
}

mod element_type_declarations {
    use strum::Display;
    use crate::topicmodel::dictionary::loader::dtd_parser::mixed_content::Mixed;

    #[derive(Debug, Clone, Hash, PartialEq, Eq, Display)]
    pub enum ContentSpec<'a> {
        #[strum(to_string="EMPTY")]
        Empty,
        #[strum(to_string="ANY")]
        Any,
        #[strum(to_string="{0}")]
        Mixed(Mixed<'a>),
        #[strum(to_string="{0}")]
        Children()
    }
}

mod element_content {
    use std::fmt::{Display, Formatter};
    use std::str::FromStr;
    use itertools::Itertools;
    use nom::bytes::complete::take;
    use nom::combinator::{map, map_res, opt};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::sequence::pair;
    use strum::{Display, EnumString};
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::Name;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Display, EnumString)]
    pub enum Multiplicity {
        #[strum(to_string="?")]
        ZeroOrOne,
        #[strum(to_string="*")]
        ZeroOrMany,
        #[strum(to_string="+")]
        OneOrMany,
    }

    pub fn multiplicity<E: ParseError<&str>>(s: &str) -> IResult<&str, Multiplicity, E> {
        map_res(take(1usize), Multiplicity::from_str)(s)
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
    pub enum InnerChildren<'a> {
        Choice(),
        Seq()
    }

    fn inner_child<E: ParseError<&str>>(s: &str) -> IResult<&str, InnerChildren, E> {
        todo!()
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct Children<'a>(
        InnerChildren<'a>,
        Option<Multiplicity>
    );

    impl<'a> Display for Children<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)?;
            if let Some(mu) = self.1 {
                write!(f, "{}", mu)?;
            }
            Ok(())
        }
    }

    pub fn children<E: ParseError<&str>>(s: &str) -> IResult<&str, Children, E> {
        map(
            pair(
                inner_child,
                opt(multiplicity)
            ),
            |(a, b)| Children(a, b)
        )(s)
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
    pub enum CP<'a> {
        Name(Name<'a>),
        Choice(Choice<'a>),
        Seq()
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct Choice<'a>(Vec<CP<'a>>);

    impl<'a> Display for Choice<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({})", self.0.iter().join("|"))
        }
    }

    pub fn choice<E: ParseError<&str>>(s: &str) -> IResult<&str, Choice, E> {

    }
}

mod mixed_content {
    use std::fmt::{write, Display, Formatter};
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{char, multispace0, multispace1};
    use nom::combinator::{map, opt, value};
    use nom::error::ParseError;
    use nom::IResult;
    use nom::multi::many1;
    use nom::sequence::{delimited, preceded, terminated};
    use crate::topicmodel::dictionary::loader::dtd_parser::names_and_tokens::Name;


    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    #[repr(transparent)]
    pub struct Mixed<'a>(Option<Vec<Name<'a>>>);

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

    pub fn mixed<E: ParseError<&str>>(s: &str) -> IResult<&str, Mixed, E> {
        map(
            alt((
                delimited(
                    terminated(char('('), multispace0),
                    preceded(
                        tag("#PCDATA"),
                        opt(
                            many1(
                                preceded(
                                    delimited(multispace0, char('|'), multispace0),
                                    Name
                                )
                            )
                        )
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
        )
    }
}

fn parse_dtd<E: ParseError<&str>>(s: &str) {
    alt((

    ))
}