use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::string::FromUtf8Error;
use itertools::Itertools;
use nom::combinator::{map, not, opt, peek, recognize, value};
use nom::error::{ParseError};
use nom::{IResult};
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag};
use nom::character::complete::{multispace0, space0, char};
use nom::multi::{many1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use thiserror::Error;
use crate::topicmodel::dictionary::loader::helper::take_bracket;
use crate::topicmodel::dictionary::loader::word_infos::{PartialWordType, WordInfo};

/// The single elements that make up an entry
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DingWordEntryElement<T> {
    Word(T),
    /// A word that is only a partial word like a prefix or a suffix
    PartialWord(T, PartialWordType),
    /// A word that alternates between the contained values
    AlternatingWords(DingAlternatingWord<T>),
    Category(T),
    Contextualisation(T),
    Info(WordInfo<T>),
    Abbreviation(T),
    AlternateNotation(T),
    /// Basically a placeholder for a word
    WordPlaceholder,
    /// Example:
    /// to put forward <> sth. -> to put forward, to put sth. forward, to put forward sth.
    InterchangeableWith,
    
}
impl<T> DingWordEntryElement<T> {
    pub fn is_word(&self) -> bool {
        match self {
            DingWordEntryElement::Word(_)
            | DingWordEntryElement::PartialWord(_, _)
            | DingWordEntryElement::AlternatingWords(_)
            | DingWordEntryElement::AlternateNotation(_)
            | DingWordEntryElement::WordPlaceholder => true,
            _ => false
        }
    }

    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingWordEntryElement<R> {
        match self {
            DingWordEntryElement::Word(value) => DingWordEntryElement::Word(mapper(value)),
            DingWordEntryElement::PartialWord(value, typ) => DingWordEntryElement::PartialWord(mapper(value), typ),
            DingWordEntryElement::Category(value) => DingWordEntryElement::Category(mapper(value)),
            DingWordEntryElement::Contextualisation(value) => DingWordEntryElement::Contextualisation(mapper(value)),
            DingWordEntryElement::Info(value) => DingWordEntryElement::Info(value.map(|value| mapper(value))),
            DingWordEntryElement::InterchangeableWith => DingWordEntryElement::InterchangeableWith,
            DingWordEntryElement::WordPlaceholder => DingWordEntryElement::WordPlaceholder,
            DingWordEntryElement::AlternatingWords(value) => DingWordEntryElement::AlternatingWords(value.map(mapper)),
            DingWordEntryElement::Abbreviation(value) => DingWordEntryElement::Abbreviation(mapper(value)),
            DingWordEntryElement::AlternateNotation(value) => DingWordEntryElement::Abbreviation(mapper(value))
        }
    }
}
impl<T> Display for DingWordEntryElement<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DingWordEntryElement::Word(value) => {
                write!(f, "{value}")
            }
            DingWordEntryElement::Abbreviation(value) => {
                write!(f, "/{value}/")
            }
            DingWordEntryElement::AlternateNotation(value) => {
                write!(f, "<{value}>")
            }
            DingWordEntryElement::Category(value) => {
                write!(f, "[{value}]")
            }
            DingWordEntryElement::Contextualisation(value) => {
                write!(f, "({value})")
            }
            DingWordEntryElement::Info(value) => {
                write!(f, "{{{value}}}")
            }
            DingWordEntryElement::InterchangeableWith => {
                write!(f, "<>")
            }
            DingWordEntryElement::WordPlaceholder => {
                write!(f, "…")
            }
            DingWordEntryElement::PartialWord(value, typ) => {
                match typ {
                    PartialWordType::Prefix => write!(f, "{value}…"),
                    PartialWordType::Suffix => write!(f, "…{value}")
                }
            }
            DingWordEntryElement::AlternatingWords(value) => {
                write!(f, "{value}")
            }
        }
    }
}

/// Represents a single element in an DingAltEntry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingAlternatingWordValue<T>(pub Vec<DingWordEntryElement<T>>);
impl<T> DingAlternatingWordValue<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingAlternatingWordValue<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingWordEntryElement<T>>> for DingAlternatingWordValue<T> {
    fn from(value: Vec<DingWordEntryElement<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingAlternatingWordValue<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" "))
    }
}

/// Represents a word that can be alternated or altered by the following entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DingAlternatingWord<T>(pub DingAlternatingWordValue<T>, pub Vec<DingAlternatingWordValue<T>>);
impl<T> DingAlternatingWord<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingAlternatingWord<R> {
        DingAlternatingWord(self.0.map(mapper), self.1.into_iter().map(|value| value.map(mapper)).collect_vec())
    }
}
impl<T> From<(DingAlternatingWordValue<T>, Vec<DingAlternatingWordValue<T>>)> for DingAlternatingWord<T> {
    fn from((leading, following): (DingAlternatingWordValue<T>, Vec<DingAlternatingWordValue<T>>)) -> Self {
        Self(leading, following)
    }
}
impl<T> Display for DingAlternatingWord<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.0, self.1.iter().join("/"))
    }
}

/// Represents a complete ding word entry
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingWordEntry<T>(pub Vec<DingWordEntryElement<T>>);
impl<T> DingWordEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingWordEntry<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingWordEntryElement<T>>> for DingWordEntry<T> {
    fn from(value: Vec<DingWordEntryElement<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingWordEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" "))
    }
}

/// Represents entries that are alternatives to each other
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingAlternativeEntries<T>(pub Vec<DingWordEntry<T>>);
impl<T> DingAlternativeEntries<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingAlternativeEntries<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingWordEntry<T>>> for DingAlternativeEntries<T> {
    fn from(value: Vec<DingWordEntry<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingAlternativeEntries<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join("; "))
    }
}

/// A collection of alternative entries
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingLanguageEntries<T>(pub Vec<DingAlternativeEntries<T>>);
impl<T> DingLanguageEntries<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingLanguageEntries<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingAlternativeEntries<T>>> for DingLanguageEntries<T> {
    fn from(value: Vec<DingAlternativeEntries<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingLanguageEntries<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" | "))
    }
}

/// An ding entry consisting of two language entries.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DingEntry<T>(pub DingLanguageEntries<T>, pub DingLanguageEntries<T>);
impl<T> DingEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> DingEntry<R> {
        DingEntry(self.0.map(&mapper), self.1.map(&mapper))
    }
}
impl<T> From<(DingLanguageEntries<T>, DingLanguageEntries<T>)> for DingEntry<T> {
    fn from((entry_a, entry_b): (DingLanguageEntries<T>, DingLanguageEntries<T>)) -> Self {
        Self(entry_a, entry_b)
    }
}
impl<T> Display for DingEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} :: {}", &self.0, &self.1)
    }
}

#[inline(always)]
fn parse_word_content<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(
        alt((
            is_not("{[(< \t:;|…/>"),
            recognize(pair(char(':'), not(char(':'))))
        ))
    )(s)
}

#[inline(always)]
fn parse_interchangeable<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    value(
        DingWordEntryElement::InterchangeableWith,
        tag("<>")
    )(s)
}

#[inline(always)]
fn parse_abbreviation<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    map(
        delimited(
            char('/'),
            delimited(space0, parse_word_content, space0),
            terminated(char('/'), peek(not(preceded(space0, parse_word))))
        ),
        DingWordEntryElement::Abbreviation
    )(s)
}

fn parse_non_word<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            map(take_bracket!('[', ']'), DingWordEntryElement::Category),
            map(take_bracket!('{', '}'), |value: &str| DingWordEntryElement::Info(value.into())),
            map(take_bracket!('(', ')'), DingWordEntryElement::Contextualisation),
            parse_abbreviation,
            map(delimited(terminated(char('<'), peek(not(char('>')))), parse_word_content, char('>')), DingWordEntryElement::AlternateNotation),
        )
    )(s)
}

fn parse_word<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            map(preceded(tag("…"), parse_word_content), |value| DingWordEntryElement::PartialWord(value, PartialWordType::Suffix)),
            value(DingWordEntryElement::WordPlaceholder, tag("…")),
            map(terminated(parse_word_content, tag("…")), |value| DingWordEntryElement::PartialWord(value, PartialWordType::Prefix)),
            map(parse_word_content, DingWordEntryElement::Word),
        )
    )(s)
}

fn parse_word_entry_element_no_alt<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            parse_non_word,
            parse_interchangeable,
            parse_word,
        )
    )(s)
}

fn parse_single_word_alternative<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingAlternatingWordValue<&'a str>, E> {
    map(
        pair(
            preceded(space0, parse_word),
            opt(
                preceded(
                    peek(preceded(space0, is_a("{[(<"))),
                    many1(
                        preceded(
                            space0,
                            parse_non_word
                        )
                    )
                )
            ),
        ),
        |(value, rest)| {
            if let Some(rest) = rest {
                let mut content = Vec::with_capacity(1+rest.len());
                content.push(value);
                content.extend(rest);
                content
            } else {
                vec![value]
            }.into()
        }
    )(s)
}


fn parse_word_alternative<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingAlternatingWord<&'a str>, E> {
    map(
        pair(
            parse_single_word_alternative,
            many1(
                delimited(
                    preceded(space0, char('/')),
                    parse_single_word_alternative,
                    space0
                )
            )
        ),
        DingAlternatingWord::from
    )(s)
}

fn parse_word_entry_element<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            map(parse_word_alternative, DingWordEntryElement::AlternatingWords),
            parse_word_entry_element_no_alt
        )
    )(s)
}

fn parse_word_entry<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntry<&'a str>, E> {
    map(
        many1(
            delimited(
                space0,
                parse_word_entry_element,
                space0
            ),
        ),
        DingWordEntry::from
    )(s)
}


fn parse_alt_word_entries<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingAlternativeEntries<&'a str>, E> {
    map(
        many1(
            alt(
                (
                    parse_word_entry,
                    preceded(
                        delimited(space0, char(';'), space0),
                        parse_word_entry
                    ),
                )
            )
        ),
        DingAlternativeEntries::from
    )(s)
}


fn parse_language_entries<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingLanguageEntries<&'a str>, E> {
    map(
        many1(
            alt(
                (
                    parse_alt_word_entries,
                    preceded(
                        delimited(space0, char('|'), space0),
                        parse_alt_word_entries
                    ),
                )
            )
        ),
        DingLanguageEntries::from
    )(s)
}

fn parse_line<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingEntry<&'a str>, E> {
    terminated(
        map(
            separated_pair(parse_language_entries, delimited(space0, tag("::"), space0), parse_language_entries),
            DingEntry::from
        ),
        multispace0
    )(s)
}

#[derive(Error, Debug)]
pub enum DingDictionaryReaderError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Parser(#[from] nom::Err<nom::error::Error<String>>),
    #[error("Failed to properly parse the data!")]
    Lost(String)
}

#[derive(Error, Debug)]
#[error("{0}: {1}")]
pub struct DingDictionaryError(
    usize,
    DingDictionaryReaderError
);


pub struct DingDictionaryReader<R> {
    reader: BufReader<R>,
    line_number: usize,
    eof: bool
}

impl<R: Read> DingDictionaryReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_number: 0,
            eof: false
        }
    }
}


impl<R: Read> DingDictionaryReader<R> {

    pub fn current_line_number(&self) -> usize {
        self.line_number
    }

    fn next_impl(&mut self) -> Option<Result<DingEntry<String>, DingDictionaryReaderError>> {
        fn parse_or_fail(content: Vec<u8>) -> Result<DingEntry<String>, DingDictionaryReaderError> {
            let content = String::from_utf8(content)?;
            let (left, entry) = parse_line::<nom::error::Error<&str>>(content.as_str()).map_err(|err| {
                match err {
                    nom::Err::Error(err) => {
                        nom::Err::Error(nom::error::Error::from_error_kind(err.input.to_string(), err.code))
                    },
                    nom::Err::Incomplete(err) => {
                        nom::Err::Incomplete(err)
                    },
                    nom::Err::Failure(err) => {
                        nom::Err::Failure(nom::error::Error::from_error_kind(err.input.to_string(), err.code))
                    }
                }
            })?;
            if !left.is_empty() {
                Err(DingDictionaryReaderError::Lost(left.to_string()))
            } else {
                Ok(entry.map(|value| value.to_string()))
            }
        }

        if self.eof {
            return None
        }
        let mut content = Vec::new();
        loop {
            match self.reader.read_until(b'\n', &mut content) {
                Ok(value) => {
                    if value == 0 {
                        self.eof = true;
                        break None
                    } else {
                        self.line_number += 1;
                        if let Some(first) = content.first() {
                            // Comment or empty line
                            if b'#'.eq(first) || b'\r'.eq(first) || b'\n'.eq(first) {
                                content.clear();
                                continue
                            }
                        }
                        break Some(parse_or_fail(content))
                    }
                }
                Err(value) => {
                    break Some(Err(value.into()))
                }
            }
        }
    }
}

impl<R: Read> Iterator for DingDictionaryReader<R> {
    type Item = Result<DingEntry<String>, DingDictionaryError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().map(|value| {
            value.map_err(|err|
                DingDictionaryError(self.line_number, err)
            )
        })
    }
}

pub fn read_dictionary(file: impl AsRef<Path>) -> io::Result<DingDictionaryReader<File>> {
    Ok(DingDictionaryReader::new(File::options().read(true).open(file)?))
}



#[cfg(test)]
mod test {
    use nom::error::VerboseError;
    use nom::Finish;
    use crate::topicmodel::dictionary::loader::ding::{parse_line, parse_word_alternative, read_dictionary};

    #[test]
    fn can_parse_alt(){
        let result = parse_word_alternative::<VerboseError<_>>("jegliche/r/s").finish();
        match &result {
            Ok(value) => {
                println!("{:?}", value.1);
            }
            Err(value) => {
                println!("{}", value.to_string());
            }
        }
    }

    #[test]
    fn can_recognize_word_category() {

        const TEST_LINES: [&'static str; 11] = [
            "Aal {m} (auf der Speisekarte) [cook.] | Aal blau; blauer Aal | Aal grün; grüner Aal | Aal in Aspik; Aal in Gelee :: Eel (on a menu) | Eel au bleu; Eel steamed and served with Butter | Boiled Eel served with Parsley Sauce | Jellied Eel",
            "A {n}; Ais {n};As {n}; Aisis {n}; Ases {n} [mus.] | A-Dur {n} :: A; A sharp; A flat; A double sharp; A double flat | A major",
            "Abbau {m}; Zersetzung {f}; Degradierung {f} (von etw.) [chem.][biol.] | bakterieller Abbau | biologischer Abbau | chemischerAbbau | photochemischer Abbau; Abbau durch Licht | metabolischer Abbau | thermischer Abbau | Abbau durch Bakterien :: breakdown; decomposition; degradation (of sth.) | bacterialdegradation | biological breakdown/degradation; biodegradation | chemical breakdown/degradation | photochemicalbreakdown/degradation; photodegradation | metabolic breakdown | thermal degradation | bacterial decomposition",
            "Ding {n}; Sache {f} | Dinge {pl}; Sachen {pl}; Krempel {m} | Dinge für sich behalten | die Dinge laufen lassen | den Dingen auf den Grund gehen | beim augenblicklichen Stand der Dinge | das Ding an sich | über solchen Dingen stehen | Er ist der Sache nicht ganz gewachsen. :: thing | things | to keep things to oneself | to let things slide | to get to the bottom of things | as things stand now; as things are now | the thing-in-itself | to be above such things | He is not really on top of things.",
            "absolut; überhaupt {adv} (Verstärkung einer Aussage) | jegliche/r/s; absolut jeder | keinerlei; absolut kein | jeglichen Zweifel ausräumen | Ich habe absolut/überhaupt keinen Grund, dorthin zurückzukehren. :: whatsoever (postpositive) (used to emphasize an assertion) | any … whatsoever | no … whatsoever | to remove any doubt whatsoever | I have no reason whatsoever to return there.; I have no reason to return there whatsoever.",
            "absondernd; sekretorisch; Sekretions…; sezernierend {adj} [biol.] | Sekretionskanälchen {n} | Sekretionsmechanismus {m} | Sekretionsnerv {n} | Gelenkschmiere sezernierend :: secretory | secretory canaliculus | secretory mechanism | secretory nerve | synoviparous",
            "alterungsbeständig {adj} (Werkstoff) {adj} :: resistant to ageing [Br.]/aging [Am.]; ageing-resistant [Br.]; aging-resistant [Am.]; non-ageing [Br.]; non-aging [Am.] (of a material)",
            "Abfallcontainer {f}; Müllcontainer {f} | Abfallcontainer {pl}; Müllcontainer {pl} :: waste/rubbish/garbage [Am.] container | waste/rubbish/garbage containers",
            "Arzneimittelnebenwirkung {f}; unerwünschte Arzeimittelwirkung {f} [pharm.] | Arzneimittelnebenwirkungen {pl}; unerwünschte Arzeimittelwirkungen {pl} | schwerwiegende Nebenwirkung; schwerwiegende unerwünschte Arzneimittelwirkung :: advserse drug reaction; adverse drug effect; adverse effect | advserse drug reactions; adverse drug effects; adverse effects | serious adverse drug reaction /SADR/; serious adverse reaction / SAR/",
            "Arztpraxis {f}; Ordination {f} [Ös.]; Arztambulatorium {n} [Südtirol] [med.] | Arztpraxen {pl}; Ordinationen {pl}; Arztambulatorien {pl} | Privatpraxis {f} | eine Arztpraxis / Ordination [Ös.] / ein Arztambulatorium [Südtirol] übernehmen :: medical practice; doctor's surgery [Br.]; medical office [Am.] | medical practices; doctor's surgeries; medical offices | private practice | to take over a medical practice/doctor's surgery [Br.] / medical office [Am.]",
            "Bereitschaftszustand {m}; Bereitschaft {f} [electr.] [techn.] | Laufzeit im Bereitschaftszustand (Mobilgeräte usw.) | Bereitschaftsverlust {m} [electr.] | im Bereitschaftsbetrieb; in Wartestellung | im Bereitschaftsmodus / in Wartestellung / einsatzbereit sein :: standby condition; standby (readiness for immediate deployment) | standby time (of mobile devices etc.) | standby loss | under standby conditions | to be on standby",
        ];
        // todo: Requires handling of alternative with ageing [Br.]/aging [Am.]
        for value in TEST_LINES {
            let result = parse_line::<VerboseError<_>>(value).finish();

            match &result {
                Ok(result) => {
                    println!("######");
                    println!("{:?}", result.1);
                    println!("-----");
                    println!("{}", value);
                    println!("-----");
                    println!("{}", result.1);
                    println!("######");
                }
                Err(value) => {
                    println!("!!!!!!");
                    println!("{}", value.to_string());
                    println!("!!!!!!");
                }
            }
        }
    }

    #[test]
    fn can_read_file(){
        let value = read_dictionary("E:\\git\\ldatranslation\\bambergdictionary\\dictionaryprocessor\\data\\ding\\de-en.txt").unwrap();
        for v in value {
            match v {
                Ok(_) => {}
                Err(error) => {
                    println!("{:?}", error)
                }
            }
        }
    }
}