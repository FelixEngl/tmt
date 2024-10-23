use std::borrow::Borrow;
use std::cell::OnceCell;
use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::{BufRead};
use std::num::ParseIntError;
use std::rc::Rc;
use std::str::{ParseBoolError, Utf8Error};
use convert_case::{Case, Casing};
use derive_builder::Builder;
use derive_more::From;
use itertools::{Either, Itertools};
use nom::{AsChar, Finish, IResult};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag_no_case, take_while1};
use nom::character::complete::{alpha1, alphanumeric0, char, multispace0, multispace1};
use nom::combinator::{all_consuming, eof, not, opt, recognize, success, value, verify};
use nom::multi::many1;
use nom::sequence::{delimited, pair, tuple};
use quick_xml::events::attributes::{AttrError, Attribute, Attributes};
use quick_xml::events::{BytesStart, Event};
use strum::{Display, EnumString};
use thiserror::Error;

pub fn analyze_xml<R: BufRead>(reader: R) -> Result<XML2CodeConverter, XML2CodeConverterError> {
    let mut data = XML2CodeConverter::default();
    data.analyze(&mut quick_xml::reader::Reader::from_reader(reader))?;
    Ok(data)
}

#[derive(Debug, Error)]
pub enum XML2CodeConverterError {
    #[error(transparent)]
    XML(#[from] quick_xml::Error),
    #[error(transparent)]
    AttributeError(#[from] AttrError),
    #[error("The attribute {1:?} has the wrong name! (expected {0})")]
    WrongAttributeName(Rc<String>, CodeAttribute),
    #[error("The element {1:?} has the wrong name! (expected {0})")]
    WrongElementName(Rc<String>, CodeElement),
    #[error("The element with the name {0} has the name already set, but tried to set the name {1}!")]
    NameAlreadySet(Rc<String>, Rc<String>),
    #[error("An illegal code element was found!")]
    IllegalRoot(CodeElement),
    #[error(transparent)]
    UTF8(#[from] Utf8Error),
    #[error("Detected multiple roots!")]
    MultipleRoots(Vec<CodeElement>),
    #[error(transparent)]
    Builder(#[from] CodeElementBuilderError),

}

#[derive(Debug, Default)]
pub struct XML2CodeConverter {
    root: OnceCell<CodeElement>
}

impl XML2CodeConverter {
    pub fn analyze<R>(&mut self, reader: &mut quick_xml::reader::Reader<R>) -> Result<(), XML2CodeConverterError> where R: BufRead {
        let mut buffer: Vec<u8> = Vec::new();
        if self.root.get().is_none() {
            self.root
                .set(CodeElement::analyze(reader, &mut buffer)?)
                .map_err(XML2CodeConverterError::IllegalRoot)?;
        } else {
            let targ = self.root.get_mut().unwrap();
            targ.register(CodeElement::analyze(reader, &mut buffer)?)?;
        }
        Ok(())
    }
}

impl Display for XML2CodeConverter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(found) = self.root.get() {
            write!(f, "{found}")
        } else {
            Ok(())
        }
    }
}


#[derive(Debug, Copy, Clone, From)]
pub enum ElementOrAttribute<'a> {
    Element(&'a CodeElement),
    Attribute(&'a CodeAttribute),
}

#[derive(Debug, Builder, Clone)]
pub struct CodeElement {
    #[builder(setter(custom))]
    real_name: Rc<String>,
    #[builder(setter(custom))]
    name: Rc<String>,
    #[builder(setter(skip), default = "1")]
    encounters: usize,
    #[builder(setter(custom), default)]
    attributes: OnceCell<HashMap<Rc<String>, CodeAttribute>>,
    #[builder(setter(custom), default)]
    elements: OnceCell<HashMap<Rc<String>, CodeElement>>,
    #[builder(setter(custom), default)]
    texts: OnceCell<Vec<String>>
}

pub struct Iter<'a> {
    target: VecDeque<ElementOrAttribute<'a>>,
}

impl<'a> Iter<'a> {
    pub fn new(start: ElementOrAttribute<'a>) -> Self {
        let mut q = VecDeque::new();
        q.push_back(start);
        Self { target: q  }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = ElementOrAttribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.target.pop_front()?;
        match target.clone() {
            ElementOrAttribute::Element(value) => {
                if let Some(values) = value.attributes.get() {
                    self.target.extend(values.values().map(ElementOrAttribute::Attribute))
                }
                if let Some(values) = value.elements.get() {
                    self.target.extend(values.values().map(ElementOrAttribute::Element))
                }
            }
            ElementOrAttribute::Attribute(_) => {}
        }
        Some(target)
    }
}

impl CodeElement {
    pub fn iter(&self) -> Iter {
        Iter::new(ElementOrAttribute::Element(self))
    }


    fn is_unique(&self) -> bool {
        self.encounters == 1
    }

    fn has_attributes(&self) -> bool {
        self.attributes.get().is_some()
    }

    fn has_elements(&self) -> bool {
        self.elements.get().is_some()
    }

    fn is_marker(&self) -> bool {
        match self.elements.get() {
            Some(values) => {
                if !values.is_empty() {
                    return false
                }
            }
            _ => {}
        }
        match self.attributes.get() {
            Some(values) => {
                if !values.is_empty() {
                    return false
                }
            }
            _ => {}
        }
        match self.texts.get() {
            None => {
                true
            }
            Some(values) => {
                values.is_empty()
            }
        }
    }

    fn get_or_init_mut<T: Default>(field: &mut OnceCell<T>) -> &mut T {
        field.get_or_init(T::default);
        field.get_mut().unwrap()
    }

    pub fn create_definition<K>(&self, map: &HashMap<K, RecognizedContentType>) -> String
    where
        K: Borrow<str> + Eq + Hash
    {
        let mut s = String::new();
        let ty = self.get_or_infer_type(map);

        use std::fmt::Write;
        let w = &mut s;

        match ty {
            Some(ContentType::Enum(ref value)) => {
                write!(w, "#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]\n").unwrap();
                write!(w, "pub enum E{value} {{").unwrap();
                for value in self.texts.get().unwrap().iter().unique() {
                    write!(w, "\n    #[strum(serialize=\"{}\")]\n    {},", value, value.to_case(Case::Pascal)).unwrap();
                }
                write!(w, "\n}}\n").unwrap();
            }
            _ => {}
        }


        write!(w, "// {}\n", self.name).unwrap();
        write!(w, "#[derive(Debug, Clone, derive_builder::Builder)]\n").unwrap();
        let name= self.type_name();
        write!(w, "pub struct {} {{\n", name).unwrap();
        if let Some(attr) = self.attributes.get() {
            for v in attr.values() {
                write!(w, "    #[builder(setter(strip_option))]\n").unwrap();
                write!(w, "    pub {}: Option<{}>,\n", v.method_base_name(), v.get_or_infer_type(map)).unwrap();
            }
        }
        let mut special_setter: Vec<&CodeElement> = Vec::new();
        if let Some(elem) = self.elements.get() {
            for v in elem.values() {
                let x = v.get_or_infer_type(map).unwrap_or(ContentType::String);
                if v.is_marker() {
                    write!(w, "    #[builder(default)]\n").unwrap();
                    write!(w, "    pub {}: bool,\n", v.method_base_name()).unwrap();
                } else if v.is_unique() {
                    write!(w, "    #[builder(setter(strip_option))]\n").unwrap();
                    write!(w, "    pub {}: Option<{}>,\n", v.method_base_name(), v.type_name()).unwrap();
                } else {
                    write!(w, "    #[builder(setter(custom))]\n").unwrap();
                    write!(w, "    pub {}: Vec<{}>,\n", v.method_base_name(), v.type_name()).unwrap();
                    special_setter.push(v);
                }
            }
        }
        if let Some(content) = ty {
            write!(w, "    #[builder(setter(strip_option))]\n").unwrap();
            match content {
                ContentType::Enum(value) => {
                    write!(w, "    pub content: Option<E{value}>,\n", ).unwrap();
                },
                other => {
                    write!(w, "    pub content: Option<{other}>,\n").unwrap();
                }
            }
        }

        write!(w, "}}").unwrap();

        if !special_setter.is_empty() {
            write!(w, "\nimpl {}Builder{{\n", name).unwrap();
            for value in special_setter {
                let m_name = value.method_base_name();
                write!(w, "    pub fn {m_name}(&mut self, value: {}){{\n", value.type_name()).unwrap();
                write!(w, "        let targ = self.{m_name}.get_or_insert_with(Default::default);\n").unwrap();
                write!(w, "        targ.push(value);\n").unwrap();
                write!(w, "    }}\n").unwrap();
            }
            write!(w, "}}").unwrap();
        }
        let tn = self.type_name();
        write!(
            w,
            "\npub fn read_{}_init<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<{}>, GenericXMLParserError>{{\n",
            self.method_base_name(),
            tn
        ).unwrap();
        write!(w, "    let mut buffer = Vec::new();\n").unwrap();
        write!(w, "    match reader.read_event_into(&mut buffer)? {{\n").unwrap();
        write!(w, "        quick_xml::events::Event::Start(start) => {{\n").unwrap();
        write!(w, "            match start.local_name().as_ref(){{\n").unwrap();
        write!(w, "                b\"{}\" => {{\n", self.real_name).unwrap();
        write!(w, "                    Ok(Some(read_{}(reader, start)?))\n", self.method_base_name()).unwrap();
        write!(w, "                }}\n").unwrap();
        write!(w, "                _ => {{Ok(None)}}\n").unwrap();
        write!(w, "            }}\n").unwrap();
        write!(w, "        }}\n").unwrap();
        write!(w, "        _ => {{Ok(None)}}\n").unwrap();
        write!(w, "    }}\n").unwrap();
        write!(w, "}}\n").unwrap();

        write!(
            w,
            "\npub fn read_{}<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<{}, GenericXMLParserError>{{\n",
            self.method_base_name(),
            tn
        ).unwrap();
        write!(w, "    let mut buffer = Vec::new();\n").unwrap();
        write!(w, "    let mut builder = {}Builder::create_empty();\n", tn).unwrap();
        if let Some(v) = self.attributes.get() {
            write!(w, "    for attr in start.attributes() {{\n").unwrap();
            write!(w, "        match attr {{\n").unwrap();
            write!(w, "            Ok(attr) => {{\n").unwrap();
            for value in v.values() {
                write!(w, "                if let Some(value) = read_{}(&attr)? {{\n", value.method_base_name()).unwrap();
                write!(w, "                    builder.{}(value);\n", value.method_base_name()).unwrap();
                write!(w, "                    continue;\n").unwrap();
                write!(w, "            }}\n").unwrap();
            }
            write!(w, "            }}\n").unwrap();
            write!(w, "            _ => {{}}\n").unwrap();
            write!(w, "        }}\n").unwrap();
            write!(w, "    }}\n").unwrap();
        }
        write!(w, "    loop{{\n").unwrap();
        write!(w, "        match reader.read_event_into(&mut buffer)? {{\n").unwrap();
        write!(w, "            quick_xml::events::Event::Start(start) => {{\n").unwrap();
        write!(w, "                match start.local_name().as_ref(){{\n").unwrap();
        if let Some(v) = self.elements.get() {
            for value in v.values() {
                write!(w, "                    b\"{}\" => {{\n", value.real_name).unwrap();
                write!(w, "                        let recognized = read_{}(reader, start)?;\n", value.method_base_name()).unwrap();
                write!(w, "                        builder.{}(recognized);\n", value.method_base_name()).unwrap();
                write!(w, "                    }}\n").unwrap();
            }
        }
        write!(w, "                    _ => {{}}\n").unwrap();
        write!(w, "                }}\n").unwrap();
        write!(w, "            }}\n").unwrap();
        write!(w, "            quick_xml::events::Event::End(value) => {{\n").unwrap();
        write!(w, "                match value.name().local_name().as_ref() {{\n").unwrap();
        write!(w, "                    b\"{}\" => {{\n", self.real_name).unwrap();
        write!(w, "                        break;\n").unwrap();
        write!(w, "                    }}\n").unwrap();
        write!(w, "                }}\n").unwrap();
        write!(w, "            }}\n").unwrap();
        write!(w, "            quick_xml::events::Event::Empty(value) => {{\n").unwrap();
        write!(w, "                \n").unwrap();
        write!(w, "                match value.local_name().as_ref(){{\n").unwrap();
        if let Some(v) = self.elements.get() {
            for value in v.values() {
                if value.is_marker() {
                    write!(w, "                    b\"{}\" => {{\n", value.real_name).unwrap();
                    write!(w, "                        builder.{}(true);\n", value.method_base_name()).unwrap();
                    write!(w, "                    }}\n").unwrap();
                } else {
                    write!(w, "                    b\"{}\" => {{\n", value.real_name).unwrap();
                    write!(w, "                        let recognized = read_{}(reader, value)?;\n", value.method_base_name()).unwrap();
                    write!(w, "                        builder.{}(recognized);\n", value.method_base_name()).unwrap();
                    write!(w, "                    }}\n").unwrap();
                }
            }
        }
        write!(w, "                    _ => {{}}\n").unwrap();
        write!(w, "                }}\n").unwrap();
        write!(w, "                break;\n").unwrap();
        write!(w, "            }}\n").unwrap();
        write!(w, "            quick_xml::events::Event::Text(value) => {{\n").unwrap();
        if let Some(typ) = self.get_or_infer_type(map) {
            write!(w, "                let s_value = std::str::from_utf8(value.as_ref())?;\n").unwrap();
            match typ {
                ContentType::String => {
                    write!(w, "                builder.content(s_value.to_string());\n").unwrap();
                }
                _ => {
                    write!(w, "                builder.content(value.trim().to_lowercase().as_str().parse()?);\n").unwrap();
                }
            }
        }
        write!(w, "            }}\n").unwrap();
        write!(w, "            quick_xml::events::Event::Eof => {{\n").unwrap();
        write!(w, "                break;\n").unwrap();
        write!(w, "            }}\n").unwrap();
        write!(w, "            _ => {{}}\n").unwrap();
        write!(w, "        }}\n").unwrap();
        write!(w, "        buffer.clear();\n").unwrap();
        write!(w, "    }}\n").unwrap();
        write!(w, "    Ok(builder.build().unwrap())\n").unwrap();
        write!(w, "}}\n").unwrap();
        s
    }

    pub fn get_or_infer_type<K>(&self, map: &HashMap<K, RecognizedContentType>) -> Option<ContentType>
    where
        K: Borrow<str> + Eq + Hash
    {
        let values = self.texts.get()?;
        if values.is_empty() {
            return None
        }
        let found = map.get(self.name.as_str()).copied().unwrap_or_else(|| RecognizedContentType::determine_type(
            values.iter().map(|value| value.as_str()).collect_vec().as_slice()
        ));
        Some(ContentType::from_recognized(found, self))
    }

    pub fn register(&mut self, other: CodeElement) -> Result<(), XML2CodeConverterError> {
        if self.real_name != other.real_name {
            Err(XML2CodeConverterError::WrongElementName(self.real_name.clone(), other))
        } else {
            self.encounters += other.encounters;
            if let Some(attr) = other.attributes.into_inner() {
                let own_attributes= Self::get_or_init_mut(&mut self.attributes);
                for (k, v) in attr.into_iter() {
                    match own_attributes.entry(k) {
                        Entry::Occupied(mut value) => {
                            value.get_mut().register(v)?;
                        }
                        Entry::Vacant(value) => {
                            value.insert(v);
                        }
                    }
                }
            }
            if let Some(elem) = other.elements.into_inner() {
                let own_elements= Self::get_or_init_mut(&mut self.elements);
                for (k, v) in elem.into_iter() {
                    match own_elements.entry(k) {
                        Entry::Occupied(mut value) => {
                            value.get_mut().register(v)?;
                        }
                        Entry::Vacant(value) => {
                            value.insert(v);
                        }
                    }
                }
            }
            if let Some(texts) = other.texts.into_inner() {
                let own_texts= Self::get_or_init_mut(&mut self.texts);
                own_texts.extend(texts);
            }
            Ok(())
        }
    }

    fn analyze_impl<'a, R>(
        reader: &mut quick_xml::reader::Reader<R>,
        outer_element: &mut CodeElementBuilder,
        buffer: &'a mut Vec<u8>,
        depth: usize
    ) -> Result<(), XML2CodeConverterError> where R: BufRead {
        let mut it = 0;
        loop {
            buffer.clear();
            match reader.read_event_into(buffer)? {
                Event::Start(start) => {
                    let mut element_inner = CodeElementBuilder::create_empty();
                    element_inner.real_name(get_real_name_as_str(&start)?)?;
                    element_inner.name(get_name_as_str(&start, depth, it)?)?;
                    it += 1;
                    element_inner.attributes(CodeAttribute::analyze_all(start.attributes(), depth)?)?;
                    Self::analyze_impl(reader, &mut element_inner, buffer, depth + 1)?;
                    outer_element.element(element_inner.build()?)?;
                }
                Event::End(_) => {
                    return Ok(())
                }
                Event::Empty(empty) => {
                    let mut element_inner = CodeElementBuilder::create_empty();
                    element_inner.real_name(get_real_name_as_str(&empty)?)?;
                    element_inner.name(get_name_as_str(&empty, depth, it)?)?;
                    it += 1;
                    element_inner.attributes(CodeAttribute::analyze_all(empty.attributes(), depth)?)?;
                    outer_element.element(element_inner.build()?)?;
                }
                Event::Text(value) => {
                    let target = std::str::from_utf8(value.as_ref())?.trim();
                    if !target.is_empty() {
                        outer_element.text(target.to_string());
                    }
                }
                Event::Eof => {
                    return Ok(())
                }
                _ => {}
            }
        }

    }

    pub fn analyze<'a, R>(reader: &mut quick_xml::reader::Reader<R>, buffer: &mut Vec<u8>) -> Result<Self, XML2CodeConverterError> where R: BufRead {
        let mut element: CodeElementBuilder = CodeElementBuilder::create_empty();
        let mut id = 0;
        loop {
            match reader.read_event_into(buffer)? {
                Event::Start(start) => {
                    element.real_name(get_real_name_as_str(&start)?)?;
                    element.name(get_name_as_str(&start, 0, id)?)?;
                    id += 1;
                    element.attributes(CodeAttribute::analyze_all(start.attributes(), 0)?)?;
                    Self::analyze_impl(reader, &mut element, buffer, 1)?;
                }
                Event::Empty(empty) => {
                    element.real_name(get_real_name_as_str(&empty)?)?;
                    element.name(get_name_as_str(&empty, 0, id)?)?;
                    id += 1;
                    element.attributes(CodeAttribute::analyze_all(empty.attributes(), 0)?)?;
                }
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }
        Ok(element.build()?)
    }


    fn write_indent(&self, f: &mut Formatter<'_>, indent_len: usize) -> std::fmt::Result {
        let indent = "  ".repeat(indent_len);
        write!(f, "{indent}E\"{}\" {{\n", self.name)?;
        write!(f, "{indent}enc:  {}\n", self.encounters)?;
        if let Some(attrs) = self.attributes.get() {
            write!(f, "{indent}attr: {}\n", attrs.len())?;
            for i in attrs.values() {
                i.write_indent(f, indent_len + 1)?;
                write!(f, "\n")?;
            }
        }
        if let Some(elem) = self.elements.get() {
            write!(f, "{indent}elem: {}\n", elem.len())?;
            for i in elem.values() {
                i.write_indent(f, indent_len + 1)?;
                write!(f, "\n")?;
            }
        }

        if let Some(txt) = self.texts.get() {
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
        self.name.to_case(Case::Pascal)
    }

    fn method_base_name(&self) -> String {
        self.name.to_case(Case::Snake)
    }
}

impl Display for CodeElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.write_indent(f, 0)
    }
}

fn get_real_name_as_str(start: &BytesStart) -> Result<String, XML2CodeConverterError> {
    Ok(std::str::from_utf8(start.name().local_name().as_ref())?.to_string())
}

fn get_name_as_str(start: &BytesStart, depth: usize, iter: usize) -> Result<String, XML2CodeConverterError> {
    Ok(format!("{}{}{}", std::str::from_utf8(start.name().local_name().as_ref())?, depth, iter))
}


impl CodeElementBuilder {
    pub fn name(&mut self, name: String) -> Result<Rc<String>, XML2CodeConverterError> {
        let name = Rc::new(name);
        match self.name.replace(name.clone()) {
            None => {
                Ok(name)
            }
            Some(value) => {
                let reset = self.name.replace(value.clone()).unwrap();
                Err(XML2CodeConverterError::NameAlreadySet(value, reset))
            }
        }
    }
    pub fn real_name(&mut self, name: String) -> Result<Rc<String>, XML2CodeConverterError> {
        let name = Rc::new(name);
        match self.real_name.replace(name.clone()) {
            None => {
                Ok(name)
            }
            Some(value) => {
                let reset = self.real_name.replace(value.clone()).unwrap();
                Err(XML2CodeConverterError::NameAlreadySet(value, reset))
            }
        }
    }

    fn init_field_and_get_mut<T: Default>(field: &mut Option<OnceCell<T>>) -> &mut T {
        let once_field = field.get_or_insert_with(Default::default);
        once_field.get_or_init(T::default);
        once_field.get_mut().unwrap()
    }


    pub fn attributes<I: IntoIterator<Item=CodeAttribute>>(&mut self, iter: I) -> Result<(), XML2CodeConverterError> {
        for value in iter {
            self.attribute(value)?;
        }
        Ok(())
    }

    pub fn attribute(&mut self, attr: CodeAttribute) -> Result<(), XML2CodeConverterError> {
        match Self::init_field_and_get_mut(&mut self.attributes).entry(attr.real_name.clone()) {
            Entry::Occupied(mut value) => {
                value.get_mut().register(attr)?;
            }
            Entry::Vacant(value) => {
                value.insert(attr);
            }
        }
        Ok(())
    }

    pub fn elements<I: IntoIterator<Item=CodeElement>>(&mut self, iter: I) -> Result<(), XML2CodeConverterError> {
        for value in iter {
            self.element(value)?;
        }
        Ok(())
    }

    pub fn element(&mut self, elem: CodeElement) -> Result<(), XML2CodeConverterError> {
        match Self::init_field_and_get_mut(&mut self.elements).entry(elem.real_name.clone()) {
            Entry::Occupied(mut value) => {
                value.get_mut().register(elem)?;
            }
            Entry::Vacant(value) => {
                value.insert(elem);
            }
        }
        Ok(())
    }

    pub fn text(&mut self, text: String) {
        Self::init_field_and_get_mut(&mut self.texts).push(text);
    }
}


#[derive(Debug, Clone)]
pub struct CodeAttribute {
    name: Rc<String>,
    real_name: Rc<String>,
    encounters: usize,
    values: HashSet<String>
}

impl CodeAttribute {
    pub fn register(&mut self, other: CodeAttribute) -> Result<(), XML2CodeConverterError> {
        if self.real_name != other.real_name {
            Err(XML2CodeConverterError::WrongAttributeName(self.real_name.clone(), other))
        } else {
            self.encounters += other.encounters;
            self.values.extend(other.values);
            Ok(())
        }
    }

    pub fn get_or_infer_type<K>(&self, map: &HashMap<K, RecognizedContentType>) -> ContentType
    where
        K: Borrow<str> + Eq + Hash
    {
        let found = map.get(self.name.as_str()).copied().unwrap_or_else(|| RecognizedContentType::determine_type(
            self.values.iter().map(|value| value.as_str()).collect_vec().as_slice()
        ));
        ContentType::from_recognized(found, self)
    }



    pub fn create_definition<K>(&self, map: &HashMap<K, RecognizedContentType>) -> String
    where
        K: Borrow<str> + Eq + Hash
    {
        let mut s = String::new();
        let ty = self.get_or_infer_type(map);
        use std::fmt::Write;
        let w = &mut s;
        match &ty {
            ContentType::Enum(en) => {
                write!(w, "#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]\n").unwrap();
                write!(w, "pub enum {en} {{").unwrap();
                for value in self.values.iter() {
                    write!(w, "\n    #[strum(serialize=\"{}\")]\n    {},", value, value.to_case(Case::Pascal)).unwrap();
                }
                write!(w, "\n}}\n").unwrap();
            }
            _ => {}
        }
        let m_name = self.method_base_name();
        write!(w, "pub fn read_{}(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<{}>, GenericXMLParserError>{{\n", m_name, ty).unwrap();
        write!(w, "    if attr.key.local_name().as_ref() == b\"{}\" {{\n", self.real_name).unwrap();
        write!(w, "        let value = attr.unescape_value()?;\n").unwrap();
        match ty {
            ContentType::String => {
                write!(w, "        Ok(Some(value.into_owned()))").unwrap();
            }
            _ => {
                write!(w, "        Ok(Some(value.trim().to_lowercase().as_str().parse()?))").unwrap();
            }
        }
        write!(w, "\n    }} else {{ Ok(None) }}").unwrap();
        write!(w, "\n}}").unwrap();
        s
    }

    pub fn analyze_all(attributes: Attributes, depth: usize) -> Result<Vec<CodeAttribute>, XML2CodeConverterError> {
        attributes.into_iter().map(|value| {
            match value {
                Ok(value) => {
                    Self::analyze_single(value, depth)
                }
                Err(value) => {
                    Err(value.into())
                }
            }
        }).collect()
    }

    pub fn analyze_single(attribute: Attribute, depth: usize) -> Result<Self, XML2CodeConverterError> {
        let name = std::str::from_utf8(attribute.key.local_name().into_inner())?;
        Ok(
            Self::new(
                name.to_string(),
                format!("{}{}", name, depth),
                attribute.unescape_value()?.into_owned(),
            )
        )
    }

    pub fn new(real_name: String, name: String, value: String) -> Self {
        let mut h = HashSet::new();
        h.insert(value);
        Self { real_name: Rc::new(real_name), name: Rc::new(name), encounters: 1, values: h }
    }

    fn write_indent(&self, f: &mut Formatter<'_>, indent_count: usize) -> std::fmt::Result {
        let indent = " ".repeat(indent_count);
        write!(f, "{indent}A\"{}\" {{\n", self.name)?;
        write!(f, "{indent}enc:  {}\n", self.encounters)?;
        write!(f, "{indent}values:\n")?;
        {
            let indent = " ".repeat(indent_count + 1);
            for attr in self.values.iter() {
                write!(f, "{indent}\"{attr}\"\n")?;
            }
        }
        write!(f, "{indent}}}")
    }
}

impl HasTypeName for CodeAttribute {
    fn type_name(&self) -> String {
        self.name.to_case(Case::Pascal)
    }

    fn method_base_name(&self) -> String {
        self.name.to_case(Case::Snake)
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


fn generate_code(converter: XML2CodeConverter) {
    let mut output = String::new();
    if let Some(targ) = converter.root.get() {

    }
}



#[derive(Debug, Error)]
pub enum GenericXMLParserError {
    #[error(transparent)]
    XML(#[from] quick_xml::Error),
    #[error(transparent)]
    AttributeError(#[from] AttrError),
    #[error(transparent)]
    BoolParser(#[from] ParseBoolError),
    #[error(transparent)]
    IntParser(#[from] ParseIntError),
    #[error(transparent)]
    StrumParser(#[from] strum::ParseError),
    #[error("The value for the field {0} is not correct {1}!")]
    IllegalValue(&'static str, String),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use crate::topicmodel::dictionary::loader::xml2code::{analyze_xml, ElementOrAttribute, GenericXMLParserError, RecognizedContentType};

    pub fn read_id(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, GenericXMLParserError>{
        if attr.key.local_name().as_ref() == b"id" {
            let value = attr.unescape_value()?;
            Ok(Some(value.into_owned()))
        } else { Ok(None) }
    }

    #[derive(Debug, Clone, derive_builder::Builder)]
    pub struct Title {
        pub content: String,
    }

    #[test]
    pub fn test(){
        // let analyzed = BufReader::new(File::open("src/topicmodel/dictionary/loader/books.xml").unwrap());
        // let result = analyze_xml(analyzed).unwrap();
        // println!("{result}");
        // let mut x = HashMap::<String, _>::new();
        // // x.insert("id".to_string(), RecognizedContentType::String);
        // x.insert("publish_date2".to_string(), RecognizedContentType::String);
        // if let Some(found) = result.root.get() {
        //     for value in found.iter(){
        //         match value {
        //             ElementOrAttribute::Element(value) => {
        //                 println!("{}", value.create_definition(&x));
        //             }
        //             ElementOrAttribute::Attribute(value) => {
        //                 println!("{}", value.create_definition(&x));
        //             }
        //         }
        //     }
        // }
        use std::io::Write;
        let mut x = HashMap::<&'static str, _>::new();
        let mut targ = BufWriter::new(File::options().write(true).open(r#"E:\git\tmt\src\topicmodel\dictionary\loader\test.rs"#).unwrap());
        let analyzed = BufReader::new(File::open(r#"D:\Downloads\freedict-eng-deu-1.9-fd1.src\eng-deu\eng-deu.tei"#).unwrap());
        let result = analyze_xml(analyzed).unwrap();
        if let Some(found) = result.root.get() {
            for value in found.iter(){
                match value {
                    ElementOrAttribute::Element(value) => {
                        write!(&mut targ, "{}\n", value.create_definition(&x)).unwrap();
                    }
                    ElementOrAttribute::Attribute(value) => {
                        write!(&mut targ, "{}", value.create_definition(&x)).unwrap();
                    }
                }
            }
        }
    }

    #[test]
    pub fn test2(){
        // use super::super::test::read_tei_0_init;
        // let x = BufReader::new(File::open(r#"D:\Downloads\freedict-eng-deu-1.9-fd1.src\eng-deu\eng-deu.tei"#).unwrap());
        // match read_tei_0_init(quick_xml::reader::Reader::from_reader(x)) {
        //     Ok(_) => {
        //         println!("Worked!")
        //     }
        //     Err(e) => {
        //         println!("Err: {e}")
        //     }
        // }
    }
}