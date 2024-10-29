use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::path::Path;
use itertools::{Itertools};
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, take_until};
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, map_parser, map_res, opt, value};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, terminated};
use strum::{Display, EnumString};
use crate::topicmodel::dictionary::loader::file_parser::{base_parser_method, FileParserResult, FunctionBasedLineWiseReader, LineWiseDictionaryReader};
use crate::topicmodel::dictionary::loader::helper::{space_only0, take_bracket, take_nested_bracket_delimited};
use crate::topicmodel::dictionary::loader::word_infos::{GrammaticalGender, PartialWordType, PartOfSpeech};
use crate::topicmodel::dictionary::word_infos::GrammaticalNumber;


/*


If you need more information than these guidelines can provide, please see the unofficial manual / FAQ document provided by Tomaquinaten at https://users.dict.cc/tomaquinaten/. It is not up to date anymore - if there are contradictions, please follow the guidelines.

Abbreviations German
{m}	der - männlich (Maskulinum)
{f}	die - weiblich (Femininum)
{n}	das - sächlich (Neutrum)
{pl}	die - Mehrzahl (Plural)

[österr.]	österreichisch
[südd.]	süddeutsch
[nordd.]	norddeutsch
[ostd.]	ostdeutsch
[schweiz.]	schweizerisch
[regional]	regional gebräuchlich (landschaftlich)

[alt]	alte Schreibweise (heute ungültig)
[ugs.]	umgangssprachlich
[fig.]	figurativ (in bildlichem, übertragenem Sinn)
[auch fig.]	auch figurativ
[Redewendung]	Redewendung
[hum.]	humoristisch
[pej.]	pejorativ (abwertend)
[vulg.]	vulgär
[veraltend]	außer Gebrauch kommend
[veraltet]	nicht mehr gebräuchlich
[geh.]	gehoben
[Rsv.]	Rechtschreibvariante
(weniger gebräuchlich)
[indekl.]	indeklinabel

jd.	jemand
jds.	jemandes
jdm.	jemandem
jdn.	jemanden
etw.	etwas
jd./etw.	jemand/etwas
jds./etw.	jemandes/etwas
jdm./etw.	jemandem/etwas
jdn./etw.	jemanden/etwas

[Gen.]	Genitiv
[Dat.]	Dativ
[Akk.]	Akkusativ
[+Gen.]	wird mit Genitiv gebraucht
[+Dat.]	wird mit Dativ gebraucht
[+Akk.]	wird mit Akkusativ gebraucht

Abbreviations English
to	for verbs
[Br.]	(esp.) British English
[Am.]	(esp.) American English

[Aus.]	Australian English
[NZ]	New Zealand English
[Can.]	Canadian English
[Scot.]	Scottish English
[Irish]	Irish English
[Ind.]	Indian English
[S.Afr.]	South African English

{pl}	to stress plural (nouns)
{sg}	to stress singular (nouns)
[attr.]	only before nouns
[postpos.]	only after nouns
[pred.]	only after verbs of being/becoming

[coll.]	colloquial
[fig.]	figurative
[also fig.]	also figurative
[idiom]	idiom
[hum.]	humorous
[pej.]	pejorative
[vulg.]	vulgar
[dated]	dated
[archaic]	archaic
[obs.]	obsolete
[literary]	literary
[spv.]	spelling variant (less common)
[sl.]	slang

sb.	somebody
sb.'s	somebody's
sth.	something
sb./sth.	somebody/something

Word Classes
adj	 adjective
adv	 adverb/adverbial
noun	 noun
verb	 verb (infinitive)
pres-p	  present participle
past-p	 past participle
prep	 preposition/adpos.
conj	 conjunction
pron	 pronoun
prefix	 prefix
suffix	 suffix
Assign all (and only those) word classes that are valid for both sides of the translation pair. forum
Subjects
see the subject list

 */

pub trait DictCCParserError<I>: ParseError<I> + FromExternalError<I, strum::ParseError>{}

impl<T, I> DictCCParserError<I> for T where T:  ParseError<I> + FromExternalError<I, strum::ParseError>{}


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum WordEntryElement<T> {
    Word(T),
    PartialWord(T, PartialWordType),
    Gender(GrammaticalGender),
    Number(GrammaticalNumber),
    MetaInfo(T),
    Contextualisation(T),
    Abbreviation(T),
    Combination(T),
    Placeholder
}

impl<T> WordEntryElement<T> {
    pub fn map<R, F: FnOnce(T) -> R>(self, mapper: F) -> WordEntryElement<R> {
        match self {
            WordEntryElement::MetaInfo(value) => WordEntryElement::MetaInfo(mapper(value)),
            WordEntryElement::Word(value) => WordEntryElement::Word(mapper(value)),
            WordEntryElement::PartialWord(value, typ) => WordEntryElement::PartialWord(mapper(value), typ),
            WordEntryElement::Gender(value) => WordEntryElement::Gender(value),
            WordEntryElement::Number(value) => WordEntryElement::Number(value),
            WordEntryElement::Contextualisation(value) => WordEntryElement::Contextualisation(mapper(value)),
            WordEntryElement::Abbreviation(value) => WordEntryElement::Abbreviation(mapper(value)),
            WordEntryElement::Combination(value) => WordEntryElement::Combination(mapper(value)),
            WordEntryElement::Placeholder => WordEntryElement::Placeholder
        }
    }
}

impl<T> Display for WordEntryElement<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WordEntryElement::Word(value) => {
                write!(f, "{value}")
            }
            WordEntryElement::Gender(value) => {
                write!(f, "{{{value}}}")
            }
            WordEntryElement::MetaInfo(value) => {
                write!(f, "{{{value}}}")
            }
            WordEntryElement::Contextualisation(value) => {
                write!(f, "[{value}]")
            }
            WordEntryElement::Abbreviation(value) => {
                write!(f, "<{value}>")
            }
            WordEntryElement::Combination(value) => {
                write!(f, "({value})")
            }
            WordEntryElement::Placeholder => {
                write!(f, "...")
            }
            WordEntryElement::PartialWord(value, typ) => {
                match typ {
                    PartialWordType::Prefix => write!(f, "{value}..."),
                    PartialWordType::Suffix => write!(f, "...{value}")
                }
            }
            WordEntryElement::Number(value) => {
                write!(f, "{{{value}}}")
            }
        }
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct WordEntry<T>(pub Vec<WordEntryElement<T>>);

impl<T> WordEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> WordEntry<R> {
        self.0.into_iter().map(|value| value.map(
            |value| mapper(value)
        )).collect_vec().into()
    }
}

impl<T> From<Vec<WordEntryElement<T>>> for WordEntry<T> {
    fn from(value: Vec<WordEntryElement<T>>) -> Self {
        Self(value)
    }
}

impl<T> Display for WordEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" "))
    }
}

#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq)]
pub enum SpecialInfo {
    #[strum(to_string = "archaic")]
    Archaic,
    #[strum(to_string = "rare")]
    Rare
}

#[derive(Clone, Debug, Copy)]
pub struct WordTypeInfo(pub Option<SpecialInfo>, pub PartOfSpeech);

impl Display for WordTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ref spec) = self.0 {
            write!(f, "{spec}:{}", self.1)
        } else {
            write!(f, "{}", self.1)
        }

    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct WordTypes(pub Vec<WordTypeInfo>);


impl Display for WordTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.iter().join(" "), f)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct WordCategories<T>(pub Vec<T>);

impl<T> WordCategories<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> WordCategories<R> {
        self.0.into_iter().map(|value| mapper(value)).collect_vec().into()
    }
}

impl<T> From<Vec<T>> for WordCategories<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> Display for WordCategories<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().join("] ["))
    }
}


#[derive(Debug, Clone)]
pub struct Entry<T>(pub WordEntry<T>, pub WordEntry<T>, pub Option<WordTypes>, pub Option<WordCategories<T>>);

impl<T> Entry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> Entry<R> {
        Entry(
            self.0.map(|value| mapper(value)),
            self.1.map(|value| mapper(value)),
            self.2,
            self.3.map(|value| value.map(|value| mapper(value))),
        )
    }
}

impl<T> From<(WordEntry<T>,WordEntry<T>, Option<WordTypes>, Option<WordCategories<T>>)> for Entry<T> {
    fn from(value: (WordEntry<T>, WordEntry<T>, Option<WordTypes>, Option<WordCategories<T>>)) -> Self {
        Entry(value.0, value.1, value.2, value.3)
    }
}

impl<T> Display for Entry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}\t", &self.0, &self.1)?;
        if let Some(ref types) = self.2 {
            write!(f, "{types}")?;
        }
        write!(f, "\t")?;
        if let Some(ref categories) = self.3 {
            write!(f, "{categories}")
        } else {
            Ok(())
        }

    }
}


fn parse_entry<'a, E: DictCCParserError<&'a str>>(s: &'a str) -> IResult<&'a str, WordEntry<&'a str>, E> {
    map(
        many1(
            delimited(
                space_only0,
                alt((
                    map(
                        take_nested_bracket_delimited('{', '}'),
                        |value: &str|
                            if let Ok(parsed) = value.parse().map(WordEntryElement::Gender) {
                                parsed
                            } else if let Ok(parsed) = value.parse().map(WordEntryElement::Number) {
                                parsed
                            } else {
                                WordEntryElement::MetaInfo(value)
                            }
                    ),
                    map(take_nested_bracket_delimited('(', ')'), WordEntryElement::Combination),
                    map(take_nested_bracket_delimited('[', ']'), WordEntryElement::Contextualisation),
                    map(take_nested_bracket_delimited('<', '>'), WordEntryElement::Abbreviation),
                    map(preceded(tag("..."), is_not("{[(< \t")), |value| WordEntryElement::PartialWord(value, PartialWordType::Suffix)),
                    value(WordEntryElement::Placeholder, tag("...")),
                    map(is_not("{[(< \t"), |value: &str| {
                        if value.ends_with("...") {
                            WordEntryElement::PartialWord(value, PartialWordType::Prefix)
                        } else {
                            WordEntryElement::Word(value)
                        }
                    }),
                )),
                space_only0
            )
        ),
        WordEntry::from
    )(s)
}


fn parse_word_type_info<'a, E: DictCCParserError<&'a str>>(s: &'a str) -> IResult<&'a str, WordTypeInfo, E> {
    map(
        pair(
            opt(terminated(map_res(take_until(":"), |value: &str| value.to_lowercase().parse()), char(':'))),
            map_res(is_not(" .\t/"), |value: &str| value.try_into()),
        ),
        |value| WordTypeInfo(value.0, value.1)
    )(s)
}

fn parse_word_type<'a, E: DictCCParserError<&'a str>>(s: &'a str) -> IResult<&'a str, WordTypes, E> {
    map(
        map_parser(
            is_not("\t"),
            separated_list0(
                is_a(" ,/"),
                parse_word_type_info
            ),
        ),
        WordTypes
    )(s)
}



fn parse_word_category<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, WordCategories<&'a str>, E> {
    map(
        many1(
            terminated(
                take_bracket!('[', ']'),
                opt(char(' '))
            )
        ),
        WordCategories
    )(s)
}


fn parse_line<'a, E: DictCCParserError<&'a str>>(s: &'a str) -> IResult<&'a str, Entry<&'a str>, E> {
    map(
        nom::sequence::tuple((
            terminated(
                parse_entry,
                char('\t')
            ),
            terminated(
                parse_entry,
                char('\t')
            ),
            terminated(
                opt(parse_word_type),
                char('\t')
            ),
            terminated(
                opt(parse_word_category),
                multispace0
            )
        )),
        Entry::from
    )(s)
}

fn parse_or_fail(content: &[u8]) -> FileParserResult<Entry<String>> {
    match base_parser_method(
        content,
        |s| parse_line::<nom::error::Error<&str>>(s)
    ) {
        Ok(value) => {
            Ok(value.map(ToString::to_string))
        }
        Err(value) => {
            Err(value.map(|value| value.map(ToString::to_string)))
        }
    }
}

pub fn read_dictionary(file: impl AsRef<Path>) -> std::io::Result<FunctionBasedLineWiseReader<File, Entry<String>>> {
    Ok(
        LineWiseDictionaryReader::new(
            File::options().read(true).open(file)?,
            parse_or_fail
        )
    )
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use itertools::Itertools;
    use nom::bytes::complete::is_not;
    use nom::IResult;
    use crate::topicmodel::dictionary::loader::dictcc;
    use crate::topicmodel::dictionary::loader::dictcc::{parse_line, parse_word_type, read_dictionary, Entry, WordCategories, WordEntryElement};
    use crate::topicmodel::dictionary::loader::helper::test::execute_test_read_for;
    use crate::topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, GrammaticalNumber, PartOfSpeech, PartialWordType, Register};
    use crate::topicmodel::dictionary::word_infos::Register::Dialect;

    #[test]
    fn word_info_parser() {
        let result: IResult<_, _> = is_not("\t")("noun	[biochem.] ");
        println!("{result:?}");
        let result: IResult<_, _> = parse_word_type("noun	[biochem.] ");
        println!("{result:?}");
    }

    #[test]
    fn can_read(){
        let value = read_dictionary(
            "dictionaries/DictCC/dict.txt"
        ).unwrap();
        execute_test_read_for(value, 30, 0);
    }

    fn process_word_entry<S: AsRef<str> + Clone>(dictcc::WordEntry(lang_cont): dictcc::WordEntry<S>) -> Vec<WordEntryElement<S>> {
        let mut gender = Vec::new();
        let mut numeric = Vec::new();
        let mut pos = Vec::new();
        let mut register = Vec::new();
        let mut abbrev = Vec::new();
        let mut domain: Vec<Domain> = Vec::new();

        let mut words = Vec::new();
        let mut prefixes = Vec::new();
        let mut postfixes = Vec::new();

        let mut unprocessed = Vec::new();

        let mut has_placeholder = false;

        for value in lang_cont {
            match value {
                WordEntryElement::Word(value) => {
                    words.push(value.as_ref().to_string())
                }
                WordEntryElement::PartialWord(a, b) => {
                    match b {
                        PartialWordType::Prefix => {
                            prefixes.push(a.as_ref().to_string())
                        }
                        PartialWordType::Suffix => {
                            postfixes.push(a.as_ref().to_string())
                        }
                    }
                }
                WordEntryElement::Gender(gend) => {
                    gender.push(gend)
                }
                WordEntryElement::Number(number) => {
                    numeric.push(number)
                }
                WordEntryElement::MetaInfo(v) => {
                    match v.as_ref() {
                        "prep+art" => {
                            pos.push(PartOfSpeech::Preposition);
                            pos.push(PartOfSpeech::Article);
                        }
                        "pron" | "adv" | "conj" | "indefinite article" => {
                            pos.push(v.as_ref().parse().unwrap())
                        }
                        "ugs." => {
                            register.push(Register::Ugs)
                        }
                        "usually pl" => {
                            numeric.push(GrammaticalNumber::Plural)
                        }
                        "sg only" => {
                            numeric.push(GrammaticalNumber::Singular)
                        }
                        "auch: f" => {
                            gender.push(GrammaticalGender::Feminine)
                        }
                        "treated as either sg or pl" | "treated as sg. or pl." => {
                            numeric.extend_from_slice(&[GrammaticalNumber::Singular, GrammaticalNumber::Plural])
                        }
                        _ => {}
                    }
                }
                ref a @ WordEntryElement::Contextualisation(ref value) => {
                    let s = value.as_ref();
                    if let Ok(reg) = s.parse() {
                        register.push(reg);
                        continue
                    }
                    if let Ok(dom) = s.parse() {
                        domain.push(dom);
                        continue
                    }
                    if matches!(
                            s,
                            "österr." | "südd." | "nordd." | "ostd." | "schweiz." | "regional"
                            | "Br." | "Am." | "Aus." | "NZ" | "Can." | "Scot." | "Irish"
                            | "Ind." | "S.Afr."
                        ) {
                        register.push(Dialect);
                        continue
                    }
                    unprocessed.push(a.clone())
                }
                WordEntryElement::Abbreviation(value) => {
                    abbrev.push(value.as_ref().to_string())
                }
                x @ WordEntryElement::Combination(_) => {
                    unprocessed.push(x)
                }
                WordEntryElement::Placeholder => {
                    // Ignore placeholders
                    has_placeholder = true;
                }
            }
        }
        unprocessed
    }

    #[test]
    fn can_read2(){
        let value = read_dictionary(
            "dictionaries/DictCC/dict.txt"
        ).unwrap();
        let mut left_over = HashSet::new();
        for val in value {
            if let Ok(Entry(
                          lang_a_cont,
                          lang_b_cont,
                          _word_types,
                          categories2
                      )) = val {
                for value in process_word_entry(lang_a_cont) {
                    left_over.insert(value);
                }
                for value in process_word_entry(lang_b_cont) {
                    left_over.insert(value);
                }
            }
        }
        let mut cont = HashSet::new();
        for value in left_over.into_iter() {
            match value {
                WordEntryElement::Word(_) => {}
                WordEntryElement::PartialWord(_, _) => {}
                WordEntryElement::Gender(_) => {}
                WordEntryElement::Number(_) => {}
                WordEntryElement::MetaInfo(_) => {}
                WordEntryElement::Contextualisation(value) => {
                    cont.insert(value);
                }
                WordEntryElement::Abbreviation(_) => {}
                WordEntryElement::Combination(_) => {}
                WordEntryElement::Placeholder => {}
            }
        }

        println!("Left Over: \n{}", cont.into_iter().take(20).join(",\n"));
    }

    #[test]
    fn read_single(){
        const VALUES: &[&str] = &[
            "&#945;-Keratin {n}	&#945;-keratin	noun	[biochem.] ",
            "(allgemeines) Besäufnis {n} [ugs.]	binge [coll.] [drinking spree]	noun	",
            "(Amerikanische) Schnappschildkröte {f}	snapper [coll.] [Chelydra serpentina]	noun	[zool.] [T] ",
            "(Echter) Alant {m}	scabwort [Inula helenium] [horse-heal]	noun	[bot.] [T] ",
            "NMR-Tomographie {f}	NMR tomography	noun	[MedTech.] ",
            "Goethe-Pflanze {f}	donkey ears {pl} [treated as sg.] [Kalanchoe pinnata, syn.: Bryophyllum calycinum, Cotyledon pinnata, Vereia pinnata]	noun	[bot.] [T] "
        ];

        for s in VALUES.iter().copied() {
            match parse_line::<nom::error::VerboseError<&str>>(s) {
                Ok((value, data)) => {
                    println!("Left: {value}\nData: {data}")
                }
                Err(value) => {
                    println!("{value:?}")
                }
            }
        }
    }
}