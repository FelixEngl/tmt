#![allow(dead_code)]
use crate::hashref::HashRef;
use convert_case::Case::Pascal;
use convert_case::{Case, Casing};
use derive_more::From;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{alpha1, alphanumeric0, char, multispace0};
use nom::combinator::{all_consuming, opt, recognize, success, value};
use nom::sequence::{delimited, pair, tuple};
use nom::{Finish, IResult};
use quick_xml::events::attributes::{AttrError, Attribute, Attributes};
use quick_xml::events::{BytesStart, Event};
use std::borrow::{Borrow};
use std::cell::OnceCell;
use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::io::BufRead;
use std::ops::{Deref};
use std::str::Utf8Error;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use strum::Display;
use thiserror::Error;

pub fn analyze_xml<R: BufRead>(reader: R) -> Result<XML2CodeConverter, XML2CodeConverterError> {
    let mut data = XML2CodeConverter::default();
    data.analyze(&mut quick_xml::reader::Reader::from_reader(reader))?;
    Ok(data)
}

#[derive(Debug, thiserror::Error)]
pub enum XML2CodeConverterError {
    #[error(transparent)]
    XML(#[from] quick_xml::Error),
    #[error(transparent)]
    AttributeError(#[from] AttrError),
    #[error("The attribute {1:?} has the wrong name! (expected {0})")]
    WrongAttributeName(HashRef<String>, CodeAttribute),
    #[error("The element {1:?} has the wrong name! (expected {0})")]
    WrongElementName(HashRef<String>, CodeElement),
    #[error("The element with the name {0} has the name already set, but tried to set the name {1}!")]
    NameAlreadySet(HashRef<String>, HashRef<String>),
    #[error("An illegal code element was found!")]
    IllegalRoot(Arc<CodeElement>),
    #[error(transparent)]
    UTF8(#[from] Utf8Error),
    #[error("Detected multiple roots!")]
    MultipleRoots(Arc<CodeElement>),
    #[error("There was no root!")]
    NoRootFound,
}

#[derive(Debug, Default)]
pub struct XML2CodeConverter {
    factory: CodeEntityFactory,
    root: OnceCell<Arc<CodeElement>>
}

impl XML2CodeConverter {

    pub fn generate_code<K, W: Write>(
        &self,
        f: &mut W,
        map: &HashMap<K, RecognizedContentType>
    ) -> std::io::Result<()>
    where
        K: Borrow<str> + Eq + Hash
    {
        let mut all_elements = Vec::new();
        if let Some(value) = self.root.get() {
            let mut builder_error_names = Vec::with_capacity(self.factory.elements.read().len());
            let error_name = format!("{}ReaderError", value.name.to_case(Pascal));
            for value in self.iter() {
                match value {
                    ElementOrAttribute::Element(value) => {
                        builder_error_names.push(value.builder_error_name());
                        write!(f, "{}\n", value.create_definition(map, &error_name)?)?;
                        all_elements.push(value);
                    }
                    ElementOrAttribute::Attribute(value) => {
                        write!(f, "{}\n", value.create_definition(map, &error_name)?)?;
                    }
                }
            }

            write!(f, r#"
pub mod iter {{
    pub trait IterHelper<I, E> {{
        fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<I>, E>;
    }}

    pub struct Iter<R, I, E, H> where H: IterHelper<I, E> {{
        reader: quick_xml::reader::Reader<R>,
        _phantom: std::marker::PhantomData<fn(H, I) -> E>
    }}

    impl<R, I, E, H> Iter<R, I, E, H>
    where
        H: IterHelper<I, E>
    {{
        pub(super) fn new(reader: quick_xml::reader::Reader<R>) -> Self {{
            Self {{ reader, _phantom: std::marker::PhantomData }}
        }}

        pub fn into_inner(self) -> quick_xml::reader::Reader<R> {{
            self.reader
        }}
    }}

    impl<R, I, E, H> Iterator for Iter<R, I, E, H>
    where
        R: std::io::BufRead,
        E: std::error::Error,
        I: Sized,
        H: IterHelper<I, E>
    {{
        type Item = Result<I, E>;

        fn next(&mut self) -> Option<Self::Item> {{
            H::goto_next(&mut self.reader).transpose()
        }}
    }}

"#)?;
            write!(f, "    use super::{};\n", error_name)?;
            for elem in all_elements {
                write!(f, "\n")?;
                write!(f, "    use super::{};\n", elem.type_name())?;
                write!(f, "    use super::{};\n", elem.generate_iter_method_wrapper_name())?;
                write!(f, "    /// Iterator for {}\n", elem.type_name())?;
                write!(f, "    pub type {} = {};\n", elem.generate_iterator_type_alias_name("R"), elem.generate_iterator_type(&error_name, "R"))?;
            }

            write!(f, "}}")?;

            write!(f, "\n\n#[derive(Debug, thiserror::Error)]\npub enum {}{{", error_name)?;
            write!(f,
                   r#"
    #[error(transparent)]
    XML(#[from] quick_xml::Error),
    #[error(transparent)]
    AttributeError(#[from] quick_xml::events::attributes::AttrError),
    #[error(transparent)]
    UTF8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    IntParserError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    FloatParserError(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    BoolParserError(#[from] std::str::ParseBoolError),
    #[error(transparent)]
    StrumParserError(#[from] strum::ParseError),
    #[error("Failed for \"{{0}}\" with {{1}} (parsed value: \"{{2}}\")")]
    AttributeStrumParserError(&'static str, strum::ParseError, String),
    #[error("Failed for \"{{0}}\" with {{1}} (parsed value: \"{{2}}\")")]
    ElementStrumParserError(&'static str, strum::ParseError, String),"#)?;
            write!(f, "\n")?;

            for name in builder_error_names {
                write!(f, "    #[error(transparent)]\n")?;
                write!(f, "    {}(#[from] {}),\n", name, name)?;
            }

            write!(f, "}}")?;
        }
        Ok(())
    }

    pub fn analyze<R>(&mut self, reader: &mut quick_xml::reader::Reader<R>) -> Result<(), XML2CodeConverterError> where R: BufRead {
        let mut buffer: Vec<u8> = Vec::new();
        if self.root.get().is_some() {
            self.analyze_root(reader, &mut buffer)?;
        } else {
            self.root
                .set(self.analyze_root(reader, &mut buffer)?)
                .map_err(XML2CodeConverterError::IllegalRoot)?;
        }
        Ok(())
    }

    fn analyze_root<R>(&self, reader: &mut quick_xml::reader::Reader<R>, buffer: &mut Vec<u8>) -> Result<Arc<CodeElement>, XML2CodeConverterError> where R: BufRead {
        let mut element: Option<Arc<CodeElement>> = None;
        loop {
            match reader.read_event_into(buffer)? {
                Event::Start(start) => {
                    if let Some(root) = element {
                        return Err(XML2CodeConverterError::MultipleRoots(root))
                    }
                    let new = self.factory.root_element(get_real_name_as_str(&start)?, 0);
                    element = Some(new.clone());
                    new.add_attributes(CodeAttribute::analyze_all(start.attributes(), &self.factory)?);
                    new.analyze(reader, buffer, 1)?;
                }
                Event::Empty(empty) => {
                    if let Some(root) = element {
                        return Err(XML2CodeConverterError::MultipleRoots(root))
                    }
                    let new = self.factory.root_element(get_real_name_as_str(&empty)?, 0);
                    element = Some(new.clone());
                    new.add_attributes(CodeAttribute::analyze_all(empty.attributes(), &self.factory)?);
                }
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }
        element.ok_or(XML2CodeConverterError::NoRootFound)
    }


    pub fn iter(&self) -> Iter {
        Iter::new(&self.factory)
    }
}

impl Display for XML2CodeConverter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(found) = self.root.get() {
            write!(f, "{}", found)
        } else {
            Ok(())
        }
    }
}


#[derive(Debug, Clone, From)]
pub enum ElementOrAttribute {
    Element(Arc<CodeElement>),
    Attribute(Arc<CodeAttribute>),
}


pub struct Iter {
    target: VecDeque<ElementOrAttribute>,
}

impl Iter {
    pub fn new(start: &CodeEntityFactory) -> Self {
        let elements = start.elements.read();
        let attributes = start.attributes.read();
        let mut dequeue = VecDeque::with_capacity(elements.len() + attributes.len());
        dequeue.extend(elements.values().cloned().map(From::from));
        dequeue.extend(attributes.values().cloned().map(From::from));
        Self { target: dequeue  }
    }
}

impl Iterator for Iter {
    type Item = ElementOrAttribute;

    fn next(&mut self) -> Option<Self::Item> {
        self.target.pop_front()
    }
}


#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct LockedRef<T>(Arc<RwLock<T>>);
unsafe impl<T> Sync for LockedRef<T>{}
unsafe impl<T> Send for LockedRef<T>{}

impl<T> LockedRef<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.0.write().unwrap()
    }
}

impl<T> Default for LockedRef<T> where T: Default {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Hash for LockedRef<T> where T: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.read().unwrap().hash(state)
    }
}

impl<T> Eq for LockedRef<T> where T: Eq {}

impl<T> PartialEq for LockedRef<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        let other = other.0.read().unwrap();
        self.0.read().unwrap().eq(other.deref())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodeEntityFactory {
    attributes: LockedRef<indexmap::IndexMap<HashRef<String>, Arc<CodeAttribute>>>,
    elements: LockedRef<indexmap::IndexMap<HashRef<String>, Arc<CodeElement>>>,
}

impl CodeEntityFactory {
    pub fn attribute(&self, name: &str) -> Arc<CodeAttribute> {
        let name = HashRef::new(name.to_string());
        let read = self.attributes.read();
        if let Some(read) = read.get(&name) {
            read.inc_encounter();
            read.clone()
        } else {
            drop(read);
            let mut write = self.attributes.write();
            match write.entry(name.clone()) {
                indexmap::map::Entry::Occupied(value) => {
                    let enc = value.get().clone();
                    enc.inc_encounter();
                    enc
                }
                indexmap::map::Entry::Vacant(value) => {
                    let value = value.insert(Arc::new(unsafe{CodeAttribute::new(name)}));
                    value.clone()
                }
            }
        }
    }

    fn get_or_create_element(&self, name: &str, depth: usize) -> Arc<CodeElement> {
        let name = HashRef::new(name.to_string());
        let read = self.elements.read();
        if let Some(element) = read.get(&name) {
            element.add_depth(depth);
            element.inc_encounters();
            element.clone()
        } else {
            drop(read);
            let mut write = self.elements.write();
            match write.entry(name.clone()) {
                indexmap::map::Entry::Occupied(value) => {
                    let enc = value.get().clone();
                    enc.add_depth(depth);
                    enc.inc_encounters();
                    enc
                }
                indexmap::map::Entry::Vacant(value) => {
                    let value = value.insert(Arc::new(unsafe{CodeElement::new(name, self.clone())}));
                    value.add_depth(depth);
                    value.clone()
                }
            }
        }
    }

    pub fn root_element(&self, name: &str, depth: usize) -> Arc<CodeElement> {
        self.get_or_create_element(name, depth)
    }

    pub fn element(&self, name: &str, parent: Arc<CodeElement>, depth: usize) -> Arc<CodeElement> {
        let created = self.get_or_create_element(name, depth);
        created.add_parent(parent);
        created
    }
}

fn unique_id() -> u64 {
    static UNIQUE_ID: AtomicU64 = AtomicU64::new(0);
    UNIQUE_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct CodeElement {
    id: u64,
    name: HashRef<String>,
    encounters_at: LockedRef<HashMap<usize, usize>>,
    encounters: Arc<AtomicUsize>,
    shared_pool: CodeEntityFactory,
    elements: LockedRef<HashMap<Arc<CodeElement>, ElementsMeta>>,
    attributes: LockedRef<HashSet<Arc<CodeAttribute>>>,
    texts: LockedRef<OnceCell<Vec<String>>>,
    parents: LockedRef<HashSet<Arc<CodeElement>>>,
    contains_self: Arc<AtomicBool>
}

#[derive(Debug, Default)]
struct ElementsMeta {
    needs_vec: bool,
    entry_to_depth_count: HashMap<usize, usize>,
    was_in_a_diff: bool
}

impl Eq for CodeElement {}

impl PartialEq for CodeElement {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for CodeElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NeededType {
    Boxed,
    Option,
    BoxedOption,
    Vec,
    NormalField
}


impl CodeElement {
    pub unsafe fn new(name: HashRef<String>, shared_pool: CodeEntityFactory) -> Self {
        Self {
            id: unique_id(),
            name,
            encounters_at: Default::default(),
            encounters: Arc::new(AtomicUsize::new(1)),
            shared_pool,
            elements: Default::default(),
            attributes: Default::default(),
            texts: Default::default(),
            parents: Default::default(),
            contains_self: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn shown_name(&self) -> String {
        format!("{}Element", self.name.to_case(Case::Pascal))
    }

    pub fn builder_error_name(&self) -> String {
        format!("{}ElementBuilderError", self.name.to_case(Case::Pascal))
    }

    pub fn add_parent(&self, parent: Arc<CodeElement>) {
        self.parents.write().insert(parent);
    }

    pub fn get_or_add_child(self: &Arc<Self>, name: &str, depth: usize) -> Arc<CodeElement> {
        let child = self.shared_pool.element(name, self.clone(), depth);
        self.add_element(child.clone(), depth);
        if self.as_ref().eq(child.as_ref()) {
            self.contains_self.store(true, Ordering::SeqCst);
        }
        child
    }

    fn add_element(&self, element: Arc<CodeElement>, depth: usize) {
        let mut writer = self.elements.write();
        match writer.entry(element) {
            Entry::Vacant(value) => {
                let mut m = ElementsMeta::default();
                m.entry_to_depth_count.insert(depth, 1);
                value.insert(m);
            }
            Entry::Occupied(mut value) => {
                *value.get_mut().entry_to_depth_count.entry(depth).or_insert(0) += 1;
            }
        }
    }

    pub fn add_attributes_raw(&self, attributes: Attributes) -> Result<(), XML2CodeConverterError> {
        let attrs = CodeAttribute::analyze_all(attributes, &self.shared_pool)?;
        self.add_attributes(attrs);
        Ok(())
    }


    fn analyze<R>(
        self: &Arc<Self>,
        reader: &mut quick_xml::reader::Reader<R>,
        buffer: &mut Vec<u8>,
        depth: usize,
    ) -> Result<(), XML2CodeConverterError> where R: BufRead {
        buffer.clear();
        let mut already_met = HashSet::new();
        let old: HashSet<_> = self.elements.read().keys().cloned().collect();
        loop {
            match reader.read_event_into(buffer)? {
                Event::Start(start) => {
                    let element_inner = self.get_or_add_child(get_real_name_as_str(&start)?, depth);
                    element_inner.add_attributes_raw(start.attributes())?;
                    element_inner.analyze(reader, buffer, depth + 1)?;
                    if !already_met.insert(element_inner.clone()) {
                        self.set_needs_vectorisation(&element_inner);
                    }
                }
                Event::End(_) => {
                    break;
                }
                Event::Empty(empty) => {
                    let element_inner = self.get_or_add_child(get_real_name_as_str(&empty)?, depth);
                    element_inner.add_attributes_raw(empty.attributes())?;
                }
                Event::Text(value) => {
                    let target = std::str::from_utf8(value.as_ref())?.trim();
                    if !target.is_empty() {
                        self.add_text(target);
                    }
                }
                Event::Eof => {
                    break;
                }
                _ => {}
            }
            buffer.clear();
        }
        for difference in old.symmetric_difference(&already_met) {
            self.register_difference(difference)
        }

        Ok(())
    }

    pub fn add_text(&self, text: impl Into<String>) {
        let mut w = self.texts.write();
        Self::get_or_init_mut(&mut w).push(text.into());
    }
    pub fn inc_encounters(&self) {
        self.encounters.fetch_add(1, Ordering::SeqCst);
    }
    pub fn add_depth(&self, value: usize) {
        *self.encounters_at.write().entry(value).or_insert(0) += 1;
    }

    pub fn add_attributes<I: IntoIterator<Item=Arc<CodeAttribute>>>(&self, attributes: I) {
        self.attributes.write().extend(attributes)
    }

    pub fn add_attribute(&self, attribute: Arc<CodeAttribute>) {
        self.attributes.write().insert(attribute);
    }



    fn get_elements_as_vec(&self) -> Vec<Arc<CodeElement>> {
        self.elements.read().keys().cloned().collect_vec()
    }

    fn get_info_to_field(&self, field: &Arc<CodeElement>) -> NeededType {
        let read = self.elements.read();
        let meta = read.get(field).expect("This is necessary!");
        if meta.needs_vec {
            return NeededType::Vec
        }
        if meta.was_in_a_diff {
            return if self.eq(field.as_ref()) {
                NeededType::BoxedOption
            } else {
                NeededType::Option
            };
        }
        let field_enc = field.encounters_at.read();
        let mut needs_option = false;
        for (depth, count_per_depth_for_field) in meta.entry_to_depth_count.iter() {
            match field_enc.get(depth) {
                None => {
                    needs_option = true
                }
                Some(count_in_whole_file_for_depth) => {
                    let count_per_depth_for_field = *count_per_depth_for_field;
                    let count_in_whole_file_for_depth = *count_in_whole_file_for_depth;
                    if count_per_depth_for_field != count_in_whole_file_for_depth {
                        needs_option = true
                    }
                }
            }
        }
        if self.eq(field.as_ref()) {
            if needs_option {
                NeededType::BoxedOption
            } else {
                NeededType::Boxed
            }
        } else {
            if needs_option {
                NeededType::Option
            } else {
                NeededType::NormalField
            }
        }
    }

    pub fn set_needs_vectorisation(&self, element: &Arc<CodeElement>) {
        let mut writer = self.elements.write();
        let value = writer.get_mut(element).expect("This value should be known at this point!");
        value.needs_vec = true;
    }


    pub fn register_difference(&self, element: &Arc<CodeElement>) {
        let mut writer = self.elements.write();
        let value = writer.get_mut(element).expect("This value should be known at this point!");
        value.was_in_a_diff = true;
    }

    fn is_unique(&self) -> bool {
        self.encounters.load(Ordering::SeqCst) == 1
    }

    fn has_attributes(&self) -> bool {
        !self.attributes.read().is_empty()
    }

    fn has_elements(&self) -> bool {
        !self.elements.read().is_empty()
    }

    fn is_marker(&self) -> bool {
        if !self.contains_self.load(Ordering::SeqCst) && self.elements.read().is_empty() && self.attributes.read().is_empty() {
            match self.texts.read().get() {
                None => {
                    true
                }
                Some(values) => {
                    values.is_empty()
                }
            }
        } else {
            false
        }
    }

    fn get_or_init_mut<T: Default>(field: &mut OnceCell<T>) -> &mut T {
        field.get_or_init(T::default);
        field.get_mut().unwrap()
    }

    fn generate_iterator_type(&self, error_name: &str, generic: &str) -> String {
        let tn = self.type_name();
        format!("Iter<{generic}, {tn}, {error_name}, {}>", self.generate_iter_method_wrapper_name())
    }

    fn generate_iterator_type_alias_name(&self, generic: &str) -> String {
        format!("{}Iter<{generic}>", self.type_name())
    }

    fn generate_iter_method_wrapper_name(&self) -> String {
        format!("{}IterFunction", self.type_name())
    }

    fn contains_self(&self) -> bool {
        self.contains_self.load(Ordering::SeqCst)
    }

    pub fn create_definition<K>(&self, map: &HashMap<K, RecognizedContentType>, error_name: &str) -> std::io::Result<String>
    where
        K: Borrow<str> + Eq + Hash
    {
        let mut s = Vec::new();
        let ty = self.get_or_infer_type(map);
        let w = &mut s;
        match ty {
            Some(ContentType::Enum(ref value)) => {
                write!(w, "#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]\n")?;
                write!(w, "pub enum E{value} {{\n")?;
                for value in self.texts.read().get().unwrap().iter().unique() {
                    write!(w, "    #[strum(serialize=\"{}\")]\n", value)?;
                    write!(w, "    {},\n", value.to_case(Case::Pascal))?;
                }
                write!(w, "}}\n\n")?;
            }
            _ => {}
        }


        write!(w, "// Element - {} - {}\n", self.name, self.name)?;
        write!(w, "#[derive(Debug, Clone, derive_builder::Builder)]\n")?;
        let name= self.type_name();
        write!(w, "pub struct {} {{\n", name)?;
        let attr_read = self.attributes.read();
        for v in attr_read.iter() {
            if v.encounters.load(Ordering::SeqCst) == self.encounters.load(Ordering::SeqCst) {
                write!(w, "    pub {}: {},\n", v.method_base_name(), v.get_or_infer_type(map))?;
            } else {
                write!(w, "    #[builder(setter(strip_option), default)]\n")?;
                write!(w, "    pub {}: Option<{}>,\n", v.method_base_name(), v.get_or_infer_type(map))?;
            }
        }
        let mut special_setter: Vec<(&Arc<CodeElement>, NeededType)> = Vec::new();
        let read_elem = self.get_elements_as_vec();
        let read_elem = read_elem.into_iter().map(
            |value| {
                let t = self.get_info_to_field(&value);
                (value, t)
            }
        ).collect_vec();

        for (read, needed_type) in read_elem.iter() {
            if self.eq(read.as_ref()) && !self.contains_self() {
                continue
            }
            if read.is_marker() {
                write!(w, "    #[builder(default)]\n")?;
                write!(w, "    pub {}: bool,\n", read.method_base_name())?;
            } else if read.is_unique() {
                write!(w, "    #[builder(setter(strip_option), default)]\n")?;
                write!(w, "    pub {}: Option<{}>,\n", read.method_base_name(), read.type_name())?;
            } else {
                match needed_type {
                    NeededType::Boxed => {
                        write!(w, "    #[builder(setter(custom))]\n")?;
                        write!(w, "    pub {}: Box<{}>,\n", read.method_base_name(), read.type_name())?;
                    }
                    NeededType::Option => {
                        write!(w, "    #[builder(setter(custom), default)]\n")?;
                        write!(w, "    pub {}: Option<{}>,\n", read.method_base_name(), read.type_name())?;
                    }
                    NeededType::BoxedOption => {
                        write!(w, "    #[builder(setter(custom), default)]\n")?;
                        write!(w, "    pub {}: Option<Box<{}>>,\n", read.method_base_name(), read.type_name())?;
                    }
                    NeededType::Vec => {
                        write!(w, "    #[builder(setter(custom), default)]\n")?;
                        write!(w, "    pub {}s: Vec<{}>,\n", read.method_base_name(), read.type_name())?;
                    }
                    NeededType::NormalField => {
                        write!(w, "    #[builder(setter(custom))]\n")?;
                        write!(w, "    pub {}: {},\n", read.method_base_name(), read.type_name())?;
                    }
                }
                special_setter.push((read, *needed_type));
            }
        }

        if let Some(content) = ty {
            let text_count = self.texts.read().get().unwrap().len();
            if text_count == self.encounters.load(Ordering::SeqCst) {
                match content {
                    ContentType::Enum(value) => {
                        write!(w, "    pub content: E{value},\n", )?;
                    },
                    other => {
                        write!(w, "    pub content: {other},\n")?;
                    }
                }
            } else {
                write!(w, "    #[builder(setter(strip_option), default)]\n")?;
                match content {
                    ContentType::Enum(value) => {
                        write!(w, "    pub content: Option<E{value}>,\n", )?;
                    },
                    other => {
                        write!(w, "    pub content: Option<{other}>,\n")?;
                    }
                }
            }
        }
        write!(w, "}}\n\n")?;

        if !special_setter.is_empty() {
            write!(w, "impl {}Builder {{\n", name)?;
            for (read, needed) in special_setter {
                if self.eq(read.as_ref()) && !self.contains_self() {
                    continue
                }
                let m_name = read.method_base_name();
                match needed {
                    NeededType::Boxed => {
                        write!(w, "    pub fn {m_name}(&mut self, value: {}){{\n", self.type_name())?;
                        write!(w, "        assert!(self.{m_name}.is_none(), \"{m_name} in {} should be unset!\");\n", self.type_name())?;
                        write!(w, "        self.{m_name} = Some(Box::new(value));\n")?;
                        write!(w, "    }}\n")?;
                    }
                    NeededType::BoxedOption => {
                        write!(w, "    pub fn {m_name}(&mut self, value: {}){{\n", self.type_name())?;
                        write!(w, "        assert!(self.{m_name}.is_none(), \"{m_name} in {} should be unset!\");\n", self.type_name())?;
                        write!(w, "        self.{m_name} = Some(Some(Box::new(value)));\n")?;
                        write!(w, "    }}\n")?;
                    }
                    NeededType::Vec => {
                        write!(w, "    pub fn {m_name}(&mut self, value: {}){{\n", read.type_name())?;
                        write!(w, "        let targ = self.{m_name}s.get_or_insert_with(Default::default);\n")?;
                        write!(w, "        targ.push(value);\n")?;
                        write!(w, "    }}\n")?;
                    }
                    NeededType::Option => {
                        write!(w, "    pub fn {m_name}(&mut self, value: {}){{\n", read.type_name())?;
                        write!(w, "        assert!(self.{m_name}.is_none(), \"{m_name} in {} should be unset!\");\n", self.type_name())?;
                        write!(w, "        self.{m_name} = Some(Some(value));\n")?;
                        write!(w, "    }}\n")?;
                    }
                    NeededType::NormalField => {
                        write!(w, "    pub fn {m_name}(&mut self, value: {}){{\n", read.type_name())?;
                        write!(w, "        assert!(self.{m_name}.is_none(), \"{m_name} in {} should be unset!\");\n", self.type_name())?;
                        write!(w, "        self.{m_name} = Some(value);\n")?;
                        write!(w, "    }}\n")?;
                    }
                }
            }
            write!(w, "}}\n\n")?;
        }
        let tn = self.type_name();
        // Iter<R, {tn}, {error_name}, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<{tn}>, {error_name}>>

        let iter_method_name = self.generate_iter_method_wrapper_name();

        write!(w, "pub struct {iter_method_name};\n\n")?;

        write!(w, "impl iter::IterHelper<{tn}, {error_name}> for {iter_method_name} {{\n")?;
        write!(w, "    #[inline(always)]\n")?;
        write!(w, "    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<{tn}>, {error_name}> {{\n")?;
        write!(w, "        read_as_root_{}(reader)\n", self.method_base_name())?;
        write!(w, "    }}\n")?;
        write!(w, "}}\n\n")?;

        write!(w, "pub fn iter_for_{}<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::{}{{\n", self.method_base_name(), self.generate_iterator_type_alias_name("R"))?;
        write!(w, "    iter::Iter::new(reader)\n")?;
        write!(w, "}}\n\n")?;

        write!(w, "pub fn read_as_root_{}<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<{}>, {error_name}>{{\n", self.method_base_name(), tn)?;
        write!(w, "    let mut buffer = Vec::new();\n")?;
        write!(w, "    loop {{\n")?;
        write!(w, "        match reader.read_event_into(&mut buffer)? {{\n")?;
        write!(w, "            quick_xml::events::Event::Start(start) => {{\n")?;
        write!(w, "                match start.local_name().as_ref(){{\n")?;
        write!(w, "                    b\"{}\" => {{\n", self.name)?;
        write!(w, "                        break Ok(Some(read_{}(reader, start)?))\n", self.method_base_name())?;
        write!(w, "                    }}\n")?;
        write!(w, "                    _ => {{}}\n")?;
        write!(w, "                }}\n")?;
        write!(w, "            }}\n")?;
        write!(w, "            quick_xml::events::Event::Eof => {{break Ok(None)}}\n")?;
        write!(w, "            _ => {{}}\n")?;
        write!(w, "        }}\n")?;
        write!(w, "        buffer.clear();\n")?;
        write!(w, "    }}\n")?;
        write!(w, "}}\n\n")?;

        if attr_read.is_empty() {
            write!(
                w,
                "pub fn read_{}<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<{}, {error_name}>{{\n",
                self.method_base_name(),
                tn
            )?;
        } else {
            write!(
                w,
                "pub fn read_{}<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<{}, {error_name}>{{\n",
                self.method_base_name(),
                tn
            )?;
        }

        write!(w, "    let mut buffer = Vec::new();\n")?;
        write!(w, "    let mut builder = {}Builder::create_empty();\n", tn)?;
        if !attr_read.is_empty() {
            write!(w, "    for attr in start.attributes() {{\n")?;
            write!(w, "        match attr {{\n")?;
            write!(w, "            Ok(attr) => {{\n")?;
            for read in attr_read.iter() {
                write!(w, "                if let Some(value) = read_{}(&attr)? {{\n", read.method_base_name())?;
                write!(w, "                    builder.{}(value);\n", read.method_base_name())?;
                write!(w, "                    continue;\n")?;
                write!(w, "                }}\n")?;
            }
            write!(w, "            }}\n")?;
            write!(w, "            _ => {{}}\n")?;
            write!(w, "        }}\n")?;
            write!(w, "    }}\n")?;
        }
        write!(w, "    loop{{\n")?;
        write!(w, "        match reader.read_event_into(&mut buffer)? {{\n")?;
        if !read_elem.is_empty() {
            write!(w, "            quick_xml::events::Event::Start(start) => {{\n")?;
            write!(w, "                match start.local_name().as_ref(){{\n")?;
            for (read, _) in read_elem.iter() {
                if self.eq(read.as_ref()) && !self.contains_self() {
                    continue
                }
                write!(w, "                    b\"{}\" => {{\n", read.name)?;
                write!(w, "                        let recognized = read_{}(reader, start)?;\n", read.method_base_name())?;
                write!(w, "                        builder.{}(recognized);\n", read.method_base_name())?;
                write!(w, "                    }}\n")?;
            }
            write!(w, "                    unknown => {{ log::warn!(\"Unknown Tag: '{{}}'\", String::from_utf8_lossy(unknown)); }}\n")?;
            write!(w, "                }}\n")?;
            write!(w, "            }}\n")?;
        }


        write!(w, "            quick_xml::events::Event::End(value) => {{\n")?;
        write!(w, "                match value.name().local_name().as_ref() {{\n")?;
        write!(w, "                    b\"{}\" => {{\n", self.name)?;
        write!(w, "                        break;\n")?;
        write!(w, "                    }}\n")?;
        write!(w, "                    _ => {{}}")?;
        write!(w, "                }}\n")?;
        write!(w, "            }}\n")?;
        if !read_elem.is_empty() {
            write!(w, "            quick_xml::events::Event::Empty(value) => {{\n")?;
            write!(w, "                \n")?;
            write!(w, "                match value.local_name().as_ref(){{\n")?;
            for value in read_elem.iter().map(|(a, _)| a) {
                let method_base_name = value.method_base_name();
                if value.is_marker() {
                    write!(w, "                    b\"{}\" => {{\n", value.name)?;
                    write!(w, "                        builder.{}(true);\n", method_base_name)?;
                    write!(w, "                    }}\n")?;
                } else {
                    write!(w, "                    b\"{}\" => {{\n", value.name)?;
                    write!(w, "                        let recognized = read_{}(reader, value)?;\n", method_base_name)?;
                    write!(w, "                        builder.{}(recognized);\n", method_base_name)?;
                    write!(w, "                    }}\n")?;
                }
            }
            write!(w, "                    unknown => {{ log::warn!(\"Unknown Tag: '{{}}'\", String::from_utf8_lossy(unknown)); }}\n")?;
            write!(w, "                }}\n")?;
            write!(w, "                break;\n")?;
            write!(w, "            }}\n")?;
        }
        if let Some(typ) = self.get_or_infer_type(map) {
            write!(w, "            quick_xml::events::Event::Text(value) => {{\n")?;
            write!(w, "                let s_value = std::str::from_utf8(value.as_ref())?;\n")?;
            match typ {
                ContentType::String => {
                    write!(w, "                builder.content(s_value.to_string());\n")?;
                }
                ContentType::Enum(_) => {
                    write!(w, "                let s = s_value.trim();\n")?;
                    write!(w, "                match s.parse(){{\n")?;
                    write!(w, "                    Ok(value) => {{\n")?;
                    write!(w, "                        builder.content(value);\n")?;
                    write!(w, "                    }}\n")?;
                    write!(w, "                    Err(error) => {{\n")?;
                    write!(w, "                        return Err({error_name}::ElementStrumParserError(\"{}\", error, s.to_string()));\n", self.name)?;
                    write!(w, "                    }}\n")?;
                    write!(w, "                }}\n")?;
                }
                _ => {
                    write!(w, "                builder.content(s_value.trim().to_lowercase().as_str().parse()?);\n")?;
                }
            }
            write!(w, "            }}\n")?;
        }
        write!(w, "            quick_xml::events::Event::Eof => {{\n")?;
        write!(w, "                break;\n")?;
        write!(w, "            }}\n")?;
        write!(w, "            _ => {{}}\n")?;
        write!(w, "        }}\n")?;
        write!(w, "        buffer.clear();\n")?;
        write!(w, "    }}\n")?;
        write!(w, "    Ok(builder.build()?)\n")?;
        write!(w, "}}\n\n")?;
        Ok(unsafe{String::from_utf8_unchecked(s)})
    }

    pub fn get_or_infer_type<K>(&self, map: &HashMap<K, RecognizedContentType>) -> Option<ContentType>
    where
        K: Borrow<str> + Eq + Hash
    {
        let values = self.texts.read();
        let values = values.get()?;
        if values.is_empty() {
            return None
        }
        let found = map.get(self.name.as_str()).copied().unwrap_or_else(|| RecognizedContentType::determine_type(
            values.iter().map(|value| value.as_str()).collect_vec().as_slice()
        ));
        Some(ContentType::from_recognized(found, self))
    }

    fn write_indent(&self, f: &mut Formatter<'_>, indent_len: usize) -> std::fmt::Result {
        let indent = "  ".repeat(indent_len);
        write!(f, "{indent}E\"{}\" {{\n", self.name)?;
        write!(f, "{indent}enc:  {}\n", self.encounters.load(Ordering::Relaxed))?;
        let attr_read = self.attributes.read();
        if !attr_read.is_empty() {
            write!(f, "{indent}attr: {}\n", attr_read.len())?;
            for i in attr_read.iter() {
                i.write_indent(f, indent_len + 1)?;
                write!(f, "\n")?;
            }
        }
        let elem_read = self.elements.read();
        if !elem_read.is_empty() {
            write!(f, "{indent}elem: {}\n", elem_read.len())?;
            for (i, value) in elem_read.iter() {
                write!(f, "{indent}  Needs Vec: {}\n", value.needs_vec)?;
                write!(f, "{indent}  Was in diff: {}\n", value.was_in_a_diff)?;
                write!(f, "{indent}  DepthCTss:\n")?;
                for (k, v) in value.entry_to_depth_count.iter() {
                    write!(f, "{indent}    {}: {}\n", *k, *v)?;
                }
                if self.eq(i.as_ref()) {
                    write!(f, "{indent}    -SELF-\n\n")?;
                    continue
                }
                i.write_indent(f, indent_len + 1)?;
                write!(f, "\n")?;
            }
        }

        if let Some(txt) = self.texts.read().get() {
            write!(f, "{indent}text: {}\n", txt.len())?;
            let indent = "  ".repeat(indent_len + 1);
            for i in txt.iter() {
                // write!(f, "{indent}\"{i}\"\n")?;
                if i.contains('\n') {
                    for value in i.split('\n') {
                        write!(f, "{indent}\"{}\"\n", value.trim())?;
                    }
                } else {
                    write!(f, "{indent}\"{}\"\n", i.trim())?;
                }
            }
        }
        write!(f, "{indent}}}")
    }
}

impl HasTypeName for CodeElement {
    fn type_name(&self) -> String {
        self.shown_name()
    }

    fn method_base_name(&self) -> String {
        self.shown_name().to_case(Case::Snake)
    }
}

impl Display for CodeElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.write_indent(f, 0)
    }
}

fn get_real_name_as_str<'a, 'b: 'a>(start: &'b BytesStart<'a>) -> Result<&'a str, XML2CodeConverterError> {
    Ok(std::str::from_utf8(start.name().local_name().into_inner())?)
}

fn get_name_as_str(start: &BytesStart, depth: usize, iter: usize) -> Result<String, XML2CodeConverterError> {
    Ok(format!("{}{}{}", std::str::from_utf8(start.name().local_name().as_ref())?, depth, iter))
}


#[derive(Debug, Clone)]
pub struct CodeAttribute {
    id: u64,
    name: HashRef<String>,
    encounters: Arc<AtomicUsize>,
    values: LockedRef<HashSet<String>>
}

impl Eq for CodeAttribute{}
impl PartialEq for CodeAttribute {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for CodeAttribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl CodeAttribute {

    pub fn shown_name(&self) -> String {
        format!("{}Attribute", self.name.to_case(Pascal))
    }

    pub fn get_or_infer_type<K>(&self, map: &HashMap<K, RecognizedContentType>) -> ContentType
    where
        K: Borrow<str> + Eq + Hash
    {
        let found = map.get(self.name.as_str()).copied().unwrap_or_else(|| RecognizedContentType::determine_type(
            self.values.read().iter().map(|value| value.as_str()).collect_vec().as_slice()
        ));
        ContentType::from_recognized(found, self)
    }

    pub fn inc_encounter(&self) {
        self.encounters.fetch_add(1, Ordering::SeqCst);
    }

    pub fn create_definition<K>(&self, map: &HashMap<K, RecognizedContentType>, error_name: &str) -> std::io::Result<String>
    where
        K: Borrow<str> + Eq + Hash
    {
        let mut s = Vec::new();
        let w = &mut s;
        let ty = self.get_or_infer_type(map);
        let name = self.shown_name();
        match &ty {
            ContentType::Enum(en) => {
                write!(w, "// Attribute - {} - {}\n", self.name, name)?;
                write!(w, "#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]\n")?;
                write!(w, "pub enum {en} {{\n")?;
                for value in self.values.read().iter() {
                    write!(w, "    #[strum(serialize=\"{}\")]\n", value)?;
                    write!(w, "    {},\n", value.to_case(Case::Pascal))?;
                }
                write!(w, "}}\n\n")?;
            }
            _ => {}
        }
        let m_name = self.method_base_name();
        write!(w, "// Attribute - {} - {}\n", self.name, name)?;
        write!(w, "pub fn read_{}(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<{}>, {error_name}>{{\n", m_name, ty)?;
        write!(w, "    if attr.key.local_name().as_ref() == b\"{}\" {{\n", self.name)?;
        write!(w, "        let value = attr.unescape_value()?;\n")?;
        match ty {
            ContentType::String => {
                write!(w, "        Ok(Some(value.into_owned()))")?;
            }
            ContentType::Enum(_) => {
                write!(w, "        let s = value.trim().to_lowercase();\n")?;
                write!(w, "        match s.parse(){{\n")?;
                write!(w, "            Ok(value) => Ok(Some(value)),\n")?;
                write!(w, "            Err(error) => Err({error_name}::AttributeStrumParserError(\"{}\", error, s)),\n", self.name)?;
                write!(w, "        }}\n")?;
            }
            _ => {
                write!(w, "        Ok(Some(value.trim().to_lowercase().as_str().parse()?))\n")?;
            }
        }
        write!(w, "    }} else {{ Ok(None) }}\n")?;
        write!(w, "}}\n")?;
        Ok(unsafe{String::from_utf8_unchecked(s)})
    }

    pub fn analyze_all(attributes: Attributes, factory: &CodeEntityFactory) -> Result<Vec<Arc<CodeAttribute>>, XML2CodeConverterError> {
        attributes.into_iter().map(|value| {
            match value {
                Ok(value) => {
                    Self::analyze_single(value, factory)
                }
                Err(value) => {
                    Err(value.into())
                }
            }
        }).collect()
    }

    pub fn analyze_single(attribute: Attribute, factory: &CodeEntityFactory) -> Result<Arc<CodeAttribute>, XML2CodeConverterError> {
        let new = factory.attribute(std::str::from_utf8(attribute.key.local_name().into_inner())?);
        new.set_value(attribute.unescape_value()?.into_owned());
        Ok(new)
    }

    pub fn set_value(&self, value: String) {
        self.values.write().insert(value);
    }

    unsafe fn new(name: HashRef<String>) -> Self {
        Self { id: unique_id(), name, encounters: Arc::new(AtomicUsize::new(1)), values: Default::default() }
    }

    fn write_indent(&self, f: &mut Formatter<'_>, indent_count: usize) -> std::fmt::Result {
        let indent = " ".repeat(indent_count);
        write!(f, "{indent}A\"{}\" {{\n", self.name)?;
        write!(f, "{indent}enc:  {}\n", self.encounters.load(Ordering::Relaxed))?;
        write!(f, "{indent}values:\n")?;
        {
            let indent = " ".repeat(indent_count + 1);
            for attr in self.values.read().iter() {
                write!(f, "{indent}\"{attr}\"\n")?;
            }
        }
        write!(f, "{indent}}}")
    }
}

impl HasTypeName for CodeAttribute {
    fn type_name(&self) -> String {
        self.shown_name()
    }

    fn method_base_name(&self) -> String {
        self.shown_name().to_case(Case::Snake)
    }
}




#[derive(Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Display, strum::EnumString)]
pub enum PrimitiveContentType {
    #[strum(to_string = "bool")]
    Bool,
    #[strum(to_string = "u64")]
    UInt,
    #[strum(to_string = "i64")]
    Int,
    #[strum(to_string = "f64")]
    Float,
}

pub trait HasTypeName {
    fn type_name(&self) -> String;
    fn method_base_name(&self) -> String;
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq, From)]
pub enum ContentType {
    Primitive(PrimitiveContentType),
    Enum(String),
    String
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Primitive(value) => {
                Display::fmt(value, f)
            }
            ContentType::Enum(value) => {
                Display::fmt(value, f)
            }
            ContentType::String => {
                write!(f, "String")
            }
        }
    }
}

impl ContentType {
    fn determine_type(data: &[&str], provider: &impl HasTypeName) -> ContentType {
        match RecognizedContentType::determine_type(data) {
            RecognizedContentType::Bool => {
                ContentType::Primitive(PrimitiveContentType::Bool)
            }
            RecognizedContentType::UInt => {
                ContentType::Primitive(PrimitiveContentType::UInt)
            }
            RecognizedContentType::Int => {
                ContentType::Primitive(PrimitiveContentType::Int)
            }
            RecognizedContentType::Float => {
                ContentType::Primitive(PrimitiveContentType::Float)
            }
            RecognizedContentType::Enum => {
                ContentType::Enum(provider.type_name())
            }
            RecognizedContentType::String => {
                ContentType::String
            }
        }
    }

    fn from_recognized(recognized: RecognizedContentType, provider: &impl HasTypeName) -> ContentType {
        match recognized {
            RecognizedContentType::Bool => {
                ContentType::Primitive(PrimitiveContentType::Bool)
            }
            RecognizedContentType::UInt => {
                ContentType::Primitive(PrimitiveContentType::UInt)
            }
            RecognizedContentType::Int => {
                ContentType::Primitive(PrimitiveContentType::Int)
            }
            RecognizedContentType::Float => {
                ContentType::Primitive(PrimitiveContentType::Float)
            }
            RecognizedContentType::Enum => {
                ContentType::Enum(provider.type_name())
            }
            RecognizedContentType::String => {
                ContentType::String
            }
        }
    }
}


#[derive(Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Display)]
pub enum RecognizedContentType {
    Bool,
    UInt,
    Int,
    Float,
    Enum,
    String
}

impl RecognizedContentType {
    fn determine_type(data: &[&str]) -> RecognizedContentType {
        if data.len() == 0 {
            return RecognizedContentType::String;
        }
        let mut target = None;
        for value2 in data {
            let recognizes: IResult<_, _, nom::error::Error<_>> = alt((
                all_consuming(delimited(multispace0, value(RecognizedContentType::Bool, alt((tag_no_case("true"), tag_no_case("false")))), multispace0)),
                all_consuming(delimited(multispace0, value(RecognizedContentType::UInt, nom::character::complete::digit1), multispace0)),
                all_consuming(delimited(multispace0, value(RecognizedContentType::Int, pair(char('-'), nom::character::complete::digit1)), multispace0)),
                all_consuming(delimited(multispace0, value(RecognizedContentType::Float, tuple((opt(char('-')), nom::character::complete::digit1, alt((char('.'), char(','))), nom::character::complete::digit1))), multispace0)),
                all_consuming(delimited(multispace0, value(RecognizedContentType::Enum, recognize(pair(alpha1, alphanumeric0))), multispace0)),
                success(RecognizedContentType::String)
            ))(*value2);

            match recognizes.finish() {
                Ok((_, other)) => {
                    if let Some(t) = target {
                        target = Some(max(t, other));
                    } else {
                        target = Some(other)
                    }
                }
                Err(_) => {}
            }
        }
        target.unwrap_or(RecognizedContentType::String)
    }
}

#[derive(Debug, Error)]
pub enum GenericXMLParserError {
    #[error(transparent)]
    XML(#[from] quick_xml::Error),
    #[error(transparent)]
    AttributeError(#[from] AttrError),
    #[error(transparent)]
    BoolParser(#[from] std::str::ParseBoolError),
    #[error(transparent)]
    IntParser(#[from] std::num::ParseIntError),
    #[error(transparent)]
    FloatParser(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    StrumParser(#[from] strum::ParseError),
    #[error("The value for the field {0} is not correct {1}!")]
    IllegalValue(&'static str, String),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}
