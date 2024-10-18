// use std::error::Error;
// use crate::topicmodel::dictionary::loader::helper::{HasLineInfo, ReadTagContentError, XmlReaderBase};
// use crate::topicmodel::dictionary::word_infos::{GrammaticalGender, GrammaticalNumber};
// use derive_builder::Builder;
// use quick_xml::events::{BytesStart, Event};
// use serde_json::de::Read;
// use std::str::Utf8Error;
// use itertools::Itertools;
// use thiserror::Error;
//
//
// impl<R> HasLineInfo for TEIReader<R> {
//     delegate::delegate! {
//         to self.inner {
//             fn current_buffer(&self) -> Option<&[u8]>;
//
//             fn current_line_number(&self) -> usize;
//         }
//     }
// }
//
// #[derive(Debug, Error)]
// pub enum TEIReaderError {
//     #[error(transparent)]
//     Xml(#[from] quick_xml::Error),
//     #[error(transparent)]
//     Utf8(#[from] Utf8Error),
//     #[error(transparent)]
//     ReadTagContent(#[from] ReadTagContentError),
//     #[error(transparent)]
//     Entry(#[from] EntryBuilderError),
//     #[error(transparent)]
//     Form(#[from] FormBuilderError)
// }
//
//
//
// #[derive(Builder)]
// pub struct Entry {
//     pub form: Form,
//     pub usage: String,
//     pub sense: Sense
// }
//
//
// #[derive(Builder)]
// pub struct GramGroup {
//     pos: String,
//     #[builder(setter(strip_option))]
//     num: Option<GrammaticalNumber>,
//     gen: Option<GrammaticalGender>,
// }
//
// #[derive(Builder)]
// pub struct Form {
//     orth: String,
//     pron: String
// }
//
// #[derive(Builder)]
// pub struct Sense {
//     #[builder(setter(custom))]
//     #[builder(field(ty = "Vec<TEICit>", build = "self.cit.clone()"))]
//     pub cit: Vec<TEICit>,
//     #[builder(default)]
//     pub note: Option<String>
// }
//
// impl SenseBuilder {
//     fn add_cit(&mut self, cit: TEICit) {
//         self.cit.push(cit)
//     }
// }
//
// #[derive(Builder)]
// pub struct TEICit {
//     type_: String,
//     quote: String,
// }
//
//
// pub struct TEIReader<R> {
//     inner: XmlReaderBase<R>,
// }
//
//
// impl<R> TEIReader<R> where R: Read {
//
//     delegate::delegate! {
//         to self.inner {
//             fn read_event(&mut self) -> Result<Event, quick_xml::Error>;
//         }
//     }
//
//
//
//     fn next_impl(&mut self) -> Result<Option<Entry>, TEIReaderError> {
//         if self.inner.is_eof() {
//             return Ok(None)
//         }
//
//
//         let mut entry_builder = EntryBuilder::create_empty();
//
//         fn create_message_for_unexpected_attributes(start: BytesStart) -> String {
//             format!("Unexpected attributes: {}: {}",
//                     String::from_utf8_lossy(start.name().as_ref()),
//                     start.attributes()
//                         .into_iter()
//                         .filter_map(|value| value.ok().map(|value| String::from_utf8_lossy(value.key.as_ref()))).join(", ")
//             )
//         }
//
//
//         loop {
//             match self.read_event()? {
//                 Event::Start(start) => {
//                     match start.name().as_ref() {
//                         b"entry" => {
//                             loop {
//                                 match self.read_event()? {
//                                     Event::Start(start) => {
//                                         match start.name().as_ref() {
//                                             b"form" => {
//                                                 if !start.attributes().next().is_none() {
//                                                     panic!(create_message_for_unexpected_attributes(start));
//                                                 }
//                                                 let mut form_builder = FormBuilder::create_empty();
//                                                 let mut current_event: Option<BytesStart<'static>> = None;
//                                                 let form = loop {
//                                                     match self.read_event()? {
//                                                         Event::Start(start) => {
//                                                             if matches!(start.name().as_ref(), b"orth" | b"pron") {
//                                                                 current_event = Some(start.into_owned())
//                                                             } else {
//                                                                 panic!("Unknown: ")
//                                                             }
//                                                         }
//                                                         Event::Text(text) => {
//                                                             match &current_event {
//                                                                 Some(start) => {
//                                                                     match start.name().as_ref() {
//                                                                         b"orth" => {
//                                                                             assert!(form_builder.orth.is_none());
//                                                                             form_builder.orth(text.unescape()?.into_owned());
//                                                                         }
//                                                                         b"pron" => {
//                                                                             assert!(form_builder.pron.is_none());
//                                                                             form_builder.pron(text.unescape()?.into_owned());
//                                                                         }
//                                                                     }
//                                                                 }
//                                                                 _ => {}
//                                                             }
//                                                         }
//                                                         Event::End(end) => {
//                                                             if matches!(end.name().as_ref(), b"form") {
//                                                                 break form_builder.build()?
//                                                             }
//                                                         }
//                                                         Event::Eof => {
//                                                             return Ok(None)
//                                                         }
//                                                         _ => {}
//                                                     }
//                                                 };
//                                                 entry_builder.form(form);
//                                             }
//                                             b"gramGrp" => {
//                                                 if !start.attributes().next().is_none() {
//                                                     panic!(create_message_for_unexpected_attributes(start));
//                                                 }
//                                                 let mut gram_grp_builder = Gram::create_empty();
//                                                 let mut current_event: Option<BytesStart<'static>> = None;
//                                             }
//                                             _ => {}
//                                         }
//                                     }
//                                     Event::End(end) => {
//                                         match end.name().as_ref() {
//                                             b"entry" => {
//                                                 return Ok(Some(entry_builder.build()?))
//                                             }
//                                         }
//                                     }
//                                     _ => {}
//                                 }
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//                 Event::Eof => {
//                     return Ok(None)
//                 }
//                 _ => {}
//             }
//         }
//
//     }
// }
