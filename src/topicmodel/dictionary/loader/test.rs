use crate::topicmodel::dictionary::loader::xml2code::GenericXMLParserError;

#[derive(Debug, Clone, derive_builder::Builder)]
// catalog0
pub struct Catalog0 {
    #[builder(setter(custom))]
    pub book_1: Vec<Book1>,
}
impl Catalog0Builder{
    pub fn set_book_1(&mut self, value: Book1){
        let targ = self.book_1.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn read_catalog_0<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<catalog0, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = Catalog0Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {
            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build()?)
}

#[derive(Debug, Clone, derive_builder::Builder)]
// book1
pub struct Book1 {
    #[builder(setter(strip_option))]
    pub id_1: Option<Id1>,
    #[builder(setter(custom))]
    pub price_2: Vec<Price2>,
    #[builder(setter(custom))]
    pub description_2: Vec<Description2>,
    #[builder(setter(custom))]
    pub author_2: Vec<Author2>,
    #[builder(setter(custom))]
    pub genre_2: Vec<Genre2>,
    #[builder(setter(custom))]
    pub title_2: Vec<Title2>,
    #[builder(setter(custom))]
    pub publish_date_2: Vec<PublishDate2>,
}
impl Book1Builder{
    pub fn set_price_2(&mut self, value: Price2){
        let targ = self.price_2.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn set_description_2(&mut self, value: Description2){
        let targ = self.description_2.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn set_author_2(&mut self, value: Author2){
        let targ = self.author_2.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn set_genre_2(&mut self, value: Genre2){
        let targ = self.genre_2.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn set_title_2(&mut self, value: Title2){
        let targ = self.title_2.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn set_publish_date_2(&mut self, value: PublishDate2){
        let targ = self.publish_date_2.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn read_book_1<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<book1, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = Book1Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {
            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build()?)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum Id1 {
    #[strum(serialize="bk107")]
    Bk107,
    #[strum(serialize="bk101")]
    Bk101,
    #[strum(serialize="bk103")]
    Bk103,
    #[strum(serialize="bk104")]
    Bk104,
    #[strum(serialize="bk109")]
    Bk109,
    #[strum(serialize="bk102")]
    Bk102,
    #[strum(serialize="bk110")]
    Bk110,
    #[strum(serialize="bk106")]
    Bk106,
    #[strum(serialize="bk108")]
    Bk108,
    #[strum(serialize="bk111")]
    Bk111,
    #[strum(serialize="bk112")]
    Bk112,
    #[strum(serialize="bk105")]
    Bk105,
}
pub fn read_id_1(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<Id1>, GenericXMLParserError>{
    if attr.key.local_name().as_ref() == b"id1" {
        let value = attr.unescape_value()?;
        Ok(Some(value.trim().to_lowercase().as_str().parse()?))
    } else { Ok(None) }
}
#[derive(Debug, Clone, derive_builder::Builder)]
// price2
pub struct Price2 {
    #[builder(setter(strip_option))]
    pub content: Option<f64>,
}
pub fn read_price_2<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<price2, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = Price2Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {
            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build()?)
}

#[derive(Debug, Clone, derive_builder::Builder)]
// description2
pub struct Description2 {
    #[builder(setter(strip_option))]
    pub content: Option<String>,
}
pub fn read_description_2<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<description2, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = Description2Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {
            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build()?)
}

#[derive(Debug, Clone, derive_builder::Builder)]
// author2
pub struct Author2 {
    #[builder(setter(strip_option))]
    pub content: Option<String>,
}
pub fn read_author_2<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<author2, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = Author2Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {
            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build()?)
}

#[derive(Debug, Clone, derive_builder::Builder)]
// genre2
pub struct Genre2 {
    #[builder(setter(strip_option))]
    pub content: Option<String>,
}
pub fn read_genre_2<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<genre2, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = Genre2Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {

            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build().unwrap())
}

#[derive(Debug, Clone, derive_builder::Builder)]
// title2
pub struct Title2 {
    #[builder(setter(strip_option))]
    pub content: Option<String>,
}
pub fn read_title_2<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<title2, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = Title2Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {
            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build()?)
}

#[derive(Debug, Clone, derive_builder::Builder)]
// publish_date2
pub struct PublishDate2 {
    #[builder(setter(strip_option))]
    pub content: Option<String>,
}
pub fn read_publish_date_2<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<PublishDate2, GenericXMLParserError>{
    let mut buffer = Vec::new();
    let mut builder = PublishDate2Builder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(value) => {
            }
            quick_xml::events::Event::End(value) => {
            }
            quick_xml::events::Event::Empty(value) => {
            }
            quick_xml::events::Event::Text(value) => {
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {
            }
        }
    }
    Ok(builder.build()?)
}

