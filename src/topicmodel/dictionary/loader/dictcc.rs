use crate::topicmodel::dictionary::loader::file_parser::{base_parser_method, FileParserResult, FunctionBasedLineWiseReader, LineWiseDictionaryReader};
use crate::topicmodel::dictionary::loader::helper::{space_only0, take_bracket, take_nested_bracket_delimited};
use crate::topicmodel::dictionary::loader::word_infos::{GrammaticalGender, PartOfSpeech, PartialWordType};
use crate::topicmodel::dictionary::word_infos::{Domain, GrammaticalNumber, Register};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, take_until};
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, map_parser, map_res, opt, value};
use nom::error::{FromExternalError, ParseError};
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{IResult, InputTake};
use regex::Regex;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::path::Path;
use std::sync::LazyLock;
use strum::{Display, EnumIs, EnumString};


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

///
/// let identifier = value.get(1);
/// let first_name = &value[2];
/// let alt_name = value.get(3);
/// let optional_middle = value.get(4);
/// let second_name = &value[5];
/// let shortage = value.get(6);
/// let third_name = value.get(7);
/// let optional_end = value.get(8);
pub static TAX_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:(also|syn\.:|formerly:) )?([A-Z][a-z]*\.?)(?: */ *)?([A-Z][a-z]*\.?)?(?: ?\(([A-Z][a-z]*\.?)\))? ([a-z]+)(?:\s([a-z]+\.))?(?:\s([a-z]+))?(?: ?\(([a-z]+)\))?"#).unwrap()
});

pub static TAX_FAMILY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:(genus|family) )?([A-Z][a-z]*)"#).unwrap()
});

#[derive(Debug, EnumIs, Clone)]
pub enum WordPatternElement<T> {
    Word(T),
    Prefix(T),
    Suffix(T),
    Combination(T),
    Placeholder
}

impl<T> Display for WordPatternElement<T> where T: AsRef<str> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WordPatternElement::Word(value) => {
                write!(f, "{}", html_escape::decode_html_entities(value))
            }
            WordPatternElement::Prefix(value) => {
                write!(f, "{}", html_escape::decode_html_entities(value))
            }
            WordPatternElement::Suffix(value) => {
                write!(f, "{}", html_escape::decode_html_entities(value))
            }
            WordPatternElement::Combination(value) => {
                write!(f, "{}", html_escape::decode_html_entities(value))
            }
            WordPatternElement::Placeholder => {
                write!(f, "…")
            }
        }
    }
}

pub struct ProcessingResult<S> {
    pub gender: Vec<GrammaticalGender>,
    pub numeric: Vec<GrammaticalNumber>,
    pub pos: Vec<PartOfSpeech>,
    pub register: Vec<Register>,
    pub abbrev: Vec<String>,
    pub domain: Vec<Domain>,
    pub synonyms: Vec<String>,
    pub word_pattern: Vec<WordPatternElement<S>>,
    pub latin_names: Vec<String>,
    pub unclassified: Vec<WordEntryElement<S>>
}

impl<S> ProcessingResult<S> where S: Clone {
    pub fn create_all_word_constructs(&self) -> Vec<Vec<WordPatternElement<S>>> {
        let mut combinations = Vec::new();
        combinations.push(vec![]);
        for value in self.word_pattern.iter() {
            match value {
                x @ WordPatternElement::Combination(_) => {
                    let mut new = Vec::with_capacity(combinations.len() * 2);
                    for targ in combinations.into_iter() {
                        let mut copy = targ.clone();
                        copy.push(x.clone());
                        new.push(copy);
                        new.push(targ);
                    }
                    combinations = new;
                }
                other => {
                    for targ in combinations.iter_mut() {
                        targ.push(other.clone());
                    }
                }
            }
        }
        combinations
    }
}

pub fn process_word_entry<S: AsRef<str> + Clone>(
    WordEntry(lang_cont): WordEntry<S>,
    additional_domains: &[Domain],
    additional_pos: &[PartOfSpeech]
) -> ProcessingResult<S> {
    let mut gender: Vec<GrammaticalGender> = Vec::new();
    let mut numeric: Vec<GrammaticalNumber> = Vec::new();
    let mut pos: Vec<PartOfSpeech> = Vec::new();
    let mut register: Vec<Register> = Vec::new();
    let mut abbrev: Vec<String> = Vec::new();
    let mut domain: Vec<Domain> = Vec::new();
    let mut synonyms: Vec<String> = Vec::new();
    let mut word_pattern: Vec<WordPatternElement<S>> = Vec::new();
    let mut latin_names: Vec<String> = Vec::new();
    let mut unclassified: Vec<WordEntryElement<S>> = Vec::new();

    for value in lang_cont {
        match value {
            WordEntryElement::Word(value) => {
                word_pattern.push(WordPatternElement::Word(value));
            }
            WordEntryElement::PartialWord(a, b) => {
                match b {
                    PartialWordType::Prefix => {
                        word_pattern.push(WordPatternElement::Prefix(a));
                    }
                    PartialWordType::Suffix => {
                        word_pattern.push(WordPatternElement::Suffix(a));
                    }
                }
            }
            WordEntryElement::Gender(gend) => {
                gender.push(gend)
            }
            WordEntryElement::Number(number) => {
                numeric.push(number)
            }
            ref a @ WordEntryElement::MetaInfo(ref v) => {
                match v.as_ref() {
                    "prep+art" => {
                        pos.push(PartOfSpeech::Preposition);
                        pos.push(PartOfSpeech::Article);
                    }
                    "pron" | "adv" | "conj" | "indefinite article" => {
                        pos.push(v.as_ref().parse().unwrap())
                    }
                    "ugs." => {
                        register.push(Register::Coll)
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
                    _ => {
                        unclassified.push(a.clone());
                    }
                }
            }
            ref a @ WordEntryElement::Contextualisation(ref value) => {

                fn parse_contextualisation(
                    s: &str,
                    register: &mut Vec<Register>,
                    domain: &mut Vec<Domain>,
                    synonyme: &mut Vec<String>,
                    latin_names: &mut Vec<String>,
                    additional_domains: &[Domain],
                    additional_pos: &[PartOfSpeech]
                ) -> bool {
                    if let Ok(reg) = s.trim_end_matches(',').parse() {
                        register.push(reg);
                        return true;
                    }
                    if let Ok(dom) = s.trim_end_matches(',').parse() {
                        domain.push(dom);
                        return true;
                    }
                    if matches!(
                            s,
                            "österr." | "südd." | "nordd." | "ostd." | "schweiz." | "regional"
                            | "Br." | "Am." | "Aus." | "NZ" | "Can." | "Scot." | "Irish"
                            | "Ind." | "S.Afr." | "westösterr."
                        ) {
                        register.push(Register::Dialect);
                        return true;
                    }

                    if s.starts_with("e.g.") || s.starts_with("z.B.") {
                        // Beispieltexte
                        return false
                    }

                    if s.starts_with("auch:") || s.starts_with("also:") || s.starts_with("syn.") {
                        let synonym = s
                            .trim_start_matches("auch:")
                            .trim_start_matches("also:")
                            .trim_start_matches("syn.:")
                            .trim();
                        if !synonym.is_empty() {
                            synonyme.push(synonym.to_string());
                            return true
                        }
                    }

                    if s.contains(' ') {
                        if additional_domains.contains(&Domain::T) && additional_pos.contains(&PartOfSpeech::Noun) {
                            let mut family_and_genus = TAX_FAMILY_REGEX.captures_iter(s).peekable();
                            if family_and_genus.peek().is_some() {
                                for value in family_and_genus {
                                    latin_names.push((&value[0]).to_string());
                                    return true
                                }
                                return true
                            }
                            drop(family_and_genus);



                            let matches = TAX_NAME_REGEX.captures_iter(s);
                            let mut matches = matches.into_iter().peekable();
                            let changed = matches.peek().is_some();
                            for value in matches {
                                // let identifier = value.get(1);
                                let first_name = &value[2];
                                let alt_name = value.get(3);
                                let optional_middle = value.get(4);
                                let second_name = &value[5];
                                let sub_definition = value.get(6);
                                let third_name = value.get(7);
                                let optional_end = value.get(8);

                                let mut entries = Vec::new();
                                entries.push(first_name.to_string());
                                if let Some(value) = alt_name {
                                    entries.push(value.as_str().to_string());
                                }
                                if let Some(optional_middle) = optional_middle {
                                    let mut collector = Vec::with_capacity(entries.len());
                                    for value in entries.iter() {
                                        let mut s = String::with_capacity(value.len() + optional_middle.as_str().len());
                                        s.push_str(value.as_str());
                                        s.push(' ');
                                        s.push_str(optional_middle.as_str());
                                        collector.push(s);
                                    }
                                    entries.extend(collector);
                                }

                                for value in entries.iter_mut() {
                                    value.push(' ');
                                    value.push_str(second_name);
                                }
                                if let Some(subs) = sub_definition {
                                    for value in entries.iter_mut() {
                                        value.push(' ');
                                        value.push_str(subs.as_str());
                                    }
                                }
                                if let Some(subs) = third_name {
                                    for value in entries.iter_mut() {
                                        value.push(' ');
                                        value.push_str(subs.as_str());
                                    }
                                }
                                if let Some(subs) = optional_end {
                                    let mut collector = Vec::with_capacity(entries.len());
                                    for value in entries.iter() {
                                        let mut s = String::with_capacity(value.len() + subs.as_str().len());
                                        s.push_str(value.as_str());
                                        s.push(' ');
                                        s.push_str(subs.as_str());
                                        collector.push(s);
                                    }
                                    entries.extend(collector);
                                }

                                latin_names.extend(entries);
                            }
                            if changed {
                                return true
                            }
                        }

                        let mut found_something = false;
                        for value in s.split(' ') {
                            found_something |= parse_contextualisation(value, register, domain, synonyme, latin_names, additional_domains, additional_pos);
                        }
                        if found_something {
                            return true
                        }
                    }
                    false
                }

                if !parse_contextualisation(value.as_ref(), &mut register, &mut domain, &mut synonyms, &mut latin_names, additional_domains, additional_pos) {
                    unclassified.push(a.clone())
                }

            }
            WordEntryElement::Abbreviation(value) => {
                abbrev.push(value.as_ref().to_string())
            }
            WordEntryElement::Combination(value) => {
                word_pattern.push(WordPatternElement::Combination(value))
            }
            WordEntryElement::Placeholder => {
                word_pattern.push(WordPatternElement::Placeholder)
            }
        }
    }
    ProcessingResult {
        gender,
        numeric,
        pos,
        register,
        abbrev,
        domain,
        synonyms,
        word_pattern,
        latin_names,
        unclassified
    }
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::dictcc;
    use crate::topicmodel::dictionary::loader::dictcc::{parse_line, parse_word_type, process_word_entry, read_dictionary, Entry, SpecialInfo, WordPatternElement, WordTypeInfo};
    use crate::topicmodel::dictionary::loader::helper::test::execute_test_read_for;
    use crate::topicmodel::dictionary::word_infos::{Domain, Register};
    use itertools::Itertools;
    use nom::bytes::complete::is_not;
    use nom::IResult;
    use rand::{seq::IteratorRandom, thread_rng};

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





    #[test]
    fn can_read2(){
        let value = read_dictionary(
            "dictionaries/DictCC/dict.txt"
        ).unwrap();
        for val in value.into_iter() {
            if let Ok(Entry(
                          lang_a_cont,
                          lang_b_cont,
                          word_types,
                          categories
                      )) = val {

                let mut general_register = Vec::new();
                let mut general_pos = Vec::new();
                if let Some(dictcc::WordTypes(types)) = word_types {
                    for WordTypeInfo(a, b) in types {
                        general_pos.push(b);
                        match a {
                            None => {}
                            Some(SpecialInfo::Archaic) => {
                                general_register.push(Register::Archaic);
                            }
                            Some(SpecialInfo::Rare) => {
                                general_register.push(Register::Rare);
                            }
                        }
                    }
                }

                let general_domains = if let Some(dictcc::WordCategories(types)) = categories {
                    types.into_iter().map(|value| value.parse::<Domain>().unwrap()).collect_vec()
                } else {
                    vec![]
                };
                let a = process_word_entry(lang_a_cont, general_domains.as_slice(), general_pos.as_slice());
                if a.word_pattern.iter().any(|value| matches!(value, WordPatternElement::Prefix(_))) {
                    println!("{:?}\n", a.word_pattern);
                }
                let a = process_word_entry(lang_b_cont, general_domains.as_slice(), general_pos.as_slice());
                if a.word_pattern.iter().any(|value| matches!(value, WordPatternElement::Prefix(_))) {
                    println!("{:?}\n", a.word_pattern);
                    break
                }

            }
        }
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