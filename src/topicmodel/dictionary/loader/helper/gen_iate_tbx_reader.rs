//hash_signature:x3bpX862vVb0MkI0vyr9Sw==

// Element - tbx - tbx
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TbxElement {
    pub xmlns_attribute: String,
    pub style_attribute: StyleAttribute,
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<String>,
    #[builder(setter(strip_option), default)]
    pub lang_attribute: Option<LangAttribute>,
    #[builder(setter(strip_option), default)]
    pub text_element: Option<TextElement>,
    #[builder(setter(strip_option), default)]
    pub tbx_header_element: Option<TbxHeaderElement>,
}

pub struct TbxElementIterFunction;

impl iter::IterHelper<TbxElement, TbxReaderError> for TbxElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TbxElement>, TbxReaderError> {
        read_as_root_tbx_element(reader)
    }
}

pub fn iter_for_tbx_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TbxElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_tbx_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TbxElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"tbx" => {
                        break Ok(Some(read_tbx_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_tbx_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TbxElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TbxElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_xmlns_attribute(&attr)? {
                    builder.xmlns_attribute(value);
                    continue;
                }
                if let Some(value) = read_style_attribute(&attr)? {
                    builder.style_attribute(value);
                    continue;
                }
                if let Some(value) = read_type_attribute(&attr)? {
                    builder.type_attribute(value);
                    continue;
                }
                if let Some(value) = read_lang_attribute(&attr)? {
                    builder.lang_attribute(value);
                    continue;
                }
            }
            _ => {}
        }
    }
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"text" => {
                        let recognized = read_text_element(reader, start)?;
                        builder.text_element(recognized);
                    }
                    b"tbxHeader" => {
                        let recognized = read_tbx_header_element(reader, start)?;
                        builder.tbx_header_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"tbx" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"text" => {
                        let recognized = read_text_element(reader, value)?;
                        builder.text_element(recognized);
                    }
                    b"tbxHeader" => {
                        let recognized = read_tbx_header_element(reader, value)?;
                        builder.tbx_header_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - tbxHeader - tbxHeader
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TbxHeaderElement {
    #[builder(setter(strip_option), default)]
    pub file_desc_element: Option<FileDescElement>,
}

pub struct TbxHeaderElementIterFunction;

impl iter::IterHelper<TbxHeaderElement, TbxReaderError> for TbxHeaderElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TbxHeaderElement>, TbxReaderError> {
        read_as_root_tbx_header_element(reader)
    }
}

pub fn iter_for_tbx_header_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TbxHeaderElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_tbx_header_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TbxHeaderElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"tbxHeader" => {
                        break Ok(Some(read_tbx_header_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_tbx_header_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TbxHeaderElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TbxHeaderElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"fileDesc" => {
                        let recognized = read_file_desc_element(reader, start)?;
                        builder.file_desc_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"tbxHeader" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"fileDesc" => {
                        let recognized = read_file_desc_element(reader, value)?;
                        builder.file_desc_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - fileDesc - fileDesc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct FileDescElement {
    #[builder(setter(strip_option), default)]
    pub title_stmt_element: Option<TitleStmtElement>,
    #[builder(setter(strip_option), default)]
    pub source_desc_element: Option<SourceDescElement>,
}

pub struct FileDescElementIterFunction;

impl iter::IterHelper<FileDescElement, TbxReaderError> for FileDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FileDescElement>, TbxReaderError> {
        read_as_root_file_desc_element(reader)
    }
}

pub fn iter_for_file_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::FileDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_file_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FileDescElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"fileDesc" => {
                        break Ok(Some(read_file_desc_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_file_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<FileDescElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = FileDescElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"titleStmt" => {
                        let recognized = read_title_stmt_element(reader, start)?;
                        builder.title_stmt_element(recognized);
                    }
                    b"sourceDesc" => {
                        let recognized = read_source_desc_element(reader, start)?;
                        builder.source_desc_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"fileDesc" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"titleStmt" => {
                        let recognized = read_title_stmt_element(reader, value)?;
                        builder.title_stmt_element(recognized);
                    }
                    b"sourceDesc" => {
                        let recognized = read_source_desc_element(reader, value)?;
                        builder.source_desc_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - titleStmt - titleStmt
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TitleStmtElement {
    #[builder(setter(strip_option), default)]
    pub title_element: Option<TitleElement>,
}

pub struct TitleStmtElementIterFunction;

impl iter::IterHelper<TitleStmtElement, TbxReaderError> for TitleStmtElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleStmtElement>, TbxReaderError> {
        read_as_root_title_stmt_element(reader)
    }
}

pub fn iter_for_title_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TitleStmtElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_title_stmt_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleStmtElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"titleStmt" => {
                        break Ok(Some(read_title_stmt_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_title_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TitleStmtElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TitleStmtElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"title" => {
                        let recognized = read_title_element(reader, start)?;
                        builder.title_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"titleStmt" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"title" => {
                        let recognized = read_title_element(reader, value)?;
                        builder.title_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - title - title
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TitleElement {
    pub content: String,
}

pub struct TitleElementIterFunction;

impl iter::IterHelper<TitleElement, TbxReaderError> for TitleElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleElement>, TbxReaderError> {
        read_as_root_title_element(reader)
    }
}

pub fn iter_for_title_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TitleElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_title_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"title" => {
                        break Ok(Some(read_title_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_title_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TitleElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TitleElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"title" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                builder.content(s_value.to_string());
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - sourceDesc - sourceDesc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct SourceDescElement {
    #[builder(setter(strip_option), default)]
    pub p_element: Option<PElement>,
}

pub struct SourceDescElementIterFunction;

impl iter::IterHelper<SourceDescElement, TbxReaderError> for SourceDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SourceDescElement>, TbxReaderError> {
        read_as_root_source_desc_element(reader)
    }
}

pub fn iter_for_source_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::SourceDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_source_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SourceDescElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"sourceDesc" => {
                        break Ok(Some(read_source_desc_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_source_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<SourceDescElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = SourceDescElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"p" => {
                        let recognized = read_p_element(reader, start)?;
                        builder.p_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"sourceDesc" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"p" => {
                        let recognized = read_p_element(reader, value)?;
                        builder.p_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - p - p
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PElement {
    pub content: String,
}

pub struct PElementIterFunction;

impl iter::IterHelper<PElement, TbxReaderError> for PElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PElement>, TbxReaderError> {
        read_as_root_p_element(reader)
    }
}

pub fn iter_for_p_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_p_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"p" => {
                        break Ok(Some(read_p_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_p_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"p" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                builder.content(s_value.to_string());
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - text - text
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TextElement {
    #[builder(setter(strip_option), default)]
    pub body_element: Option<BodyElement>,
}

pub struct TextElementIterFunction;

impl iter::IterHelper<TextElement, TbxReaderError> for TextElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TextElement>, TbxReaderError> {
        read_as_root_text_element(reader)
    }
}

pub fn iter_for_text_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TextElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_text_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TextElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"text" => {
                        break Ok(Some(read_text_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_text_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TextElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TextElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"body" => {
                        let recognized = read_body_element(reader, start)?;
                        builder.body_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"text" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"body" => {
                        let recognized = read_body_element(reader, value)?;
                        builder.body_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - body - body
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct BodyElement {
    #[builder(setter(custom), default)]
    pub concept_entry_elements: Vec<ConceptEntryElement>,
}

impl BodyElementBuilder {
    pub fn concept_entry_element(&mut self, value: ConceptEntryElement){
        let targ = self.concept_entry_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct BodyElementIterFunction;

impl iter::IterHelper<BodyElement, TbxReaderError> for BodyElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<BodyElement>, TbxReaderError> {
        read_as_root_body_element(reader)
    }
}

pub fn iter_for_body_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::BodyElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_body_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<BodyElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"body" => {
                        break Ok(Some(read_body_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_body_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<BodyElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = BodyElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"conceptEntry" => {
                        let recognized = read_concept_entry_element(reader, start)?;
                        builder.concept_entry_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"body" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"conceptEntry" => {
                        let recognized = read_concept_entry_element(reader, value)?;
                        builder.concept_entry_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - conceptEntry - conceptEntry
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct ConceptEntryElement {
    pub id_attribute: u64,
    #[builder(setter(custom), default)]
    pub lang_sec_elements: Vec<LangSecElement>,
    #[builder(setter(custom), default)]
    pub descrip_element: Option<DescripElement>,
}

impl ConceptEntryElementBuilder {
    pub fn lang_sec_element(&mut self, value: LangSecElement){
        let targ = self.lang_sec_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn descrip_element(&mut self, value: DescripElement){
        assert!(self.descrip_element.is_none(), "descrip_element in ConceptEntryElement should be unset!");
        self.descrip_element = Some(Some(value));
    }
}

pub struct ConceptEntryElementIterFunction;

impl iter::IterHelper<ConceptEntryElement, TbxReaderError> for ConceptEntryElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ConceptEntryElement>, TbxReaderError> {
        read_as_root_concept_entry_element(reader)
    }
}

pub fn iter_for_concept_entry_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::ConceptEntryElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_concept_entry_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ConceptEntryElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"conceptEntry" => {
                        break Ok(Some(read_concept_entry_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_concept_entry_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<ConceptEntryElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = ConceptEntryElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_id_attribute(&attr)? {
                    builder.id_attribute(value);
                    continue;
                }
            }
            _ => {}
        }
    }
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"langSec" => {
                        let recognized = read_lang_sec_element(reader, start)?;
                        builder.lang_sec_element(recognized);
                    }
                    b"descrip" => {
                        let recognized = read_descrip_element(reader, start)?;
                        builder.descrip_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"conceptEntry" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"langSec" => {
                        let recognized = read_lang_sec_element(reader, value)?;
                        builder.lang_sec_element(recognized);
                    }
                    b"descrip" => {
                        let recognized = read_descrip_element(reader, value)?;
                        builder.descrip_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - descrip - descrip
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct DescripElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<String>,
    pub content: String,
}

pub struct DescripElementIterFunction;

impl iter::IterHelper<DescripElement, TbxReaderError> for DescripElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DescripElement>, TbxReaderError> {
        read_as_root_descrip_element(reader)
    }
}

pub fn iter_for_descrip_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::DescripElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_descrip_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DescripElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"descrip" => {
                        break Ok(Some(read_descrip_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_descrip_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<DescripElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = DescripElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_type_attribute(&attr)? {
                    builder.type_attribute(value);
                    continue;
                }
            }
            _ => {}
        }
    }
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"descrip" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                builder.content(s_value.to_string());
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - langSec - langSec
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct LangSecElement {
    #[builder(setter(strip_option), default)]
    pub lang_attribute: Option<LangAttribute>,
    #[builder(setter(custom), default)]
    pub term_sec_elements: Vec<TermSecElement>,
}

impl LangSecElementBuilder {
    pub fn term_sec_element(&mut self, value: TermSecElement){
        let targ = self.term_sec_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct LangSecElementIterFunction;

impl iter::IterHelper<LangSecElement, TbxReaderError> for LangSecElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<LangSecElement>, TbxReaderError> {
        read_as_root_lang_sec_element(reader)
    }
}

pub fn iter_for_lang_sec_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::LangSecElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_lang_sec_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<LangSecElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"langSec" => {
                        break Ok(Some(read_lang_sec_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_lang_sec_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<LangSecElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = LangSecElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_lang_attribute(&attr)? {
                    builder.lang_attribute(value);
                    continue;
                }
            }
            _ => {}
        }
    }
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termSec" => {
                        let recognized = read_term_sec_element(reader, start)?;
                        builder.term_sec_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"langSec" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"termSec" => {
                        let recognized = read_term_sec_element(reader, value)?;
                        builder.term_sec_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - termSec - termSec
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TermSecElement {
    #[builder(setter(custom), default)]
    pub term_note_elements: Vec<TermNoteElement>,
    #[builder(setter(custom), default)]
    pub term_element: Option<TermElement>,
    #[builder(setter(custom), default)]
    pub descrip_element: Option<DescripElement>,
}

impl TermSecElementBuilder {
    pub fn term_note_element(&mut self, value: TermNoteElement){
        let targ = self.term_note_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn term_element(&mut self, value: TermElement){
        assert!(self.term_element.is_none(), "term_element in TermSecElement should be unset!");
        self.term_element = Some(Some(value));
    }
    pub fn descrip_element(&mut self, value: DescripElement){
        assert!(self.descrip_element.is_none(), "descrip_element in TermSecElement should be unset!");
        self.descrip_element = Some(Some(value));
    }
}

pub struct TermSecElementIterFunction;

impl iter::IterHelper<TermSecElement, TbxReaderError> for TermSecElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermSecElement>, TbxReaderError> {
        read_as_root_term_sec_element(reader)
    }
}

pub fn iter_for_term_sec_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TermSecElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_term_sec_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermSecElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termSec" => {
                        break Ok(Some(read_term_sec_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_term_sec_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TermSecElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TermSecElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termNote" => {
                        let recognized = read_term_note_element(reader, start)?;
                        builder.term_note_element(recognized);
                    }
                    b"term" => {
                        let recognized = read_term_element(reader, start)?;
                        builder.term_element(recognized);
                    }
                    b"descrip" => {
                        let recognized = read_descrip_element(reader, start)?;
                        builder.descrip_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"termSec" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"termNote" => {
                        let recognized = read_term_note_element(reader, value)?;
                        builder.term_note_element(recognized);
                    }
                    b"term" => {
                        let recognized = read_term_element(reader, value)?;
                        builder.term_element(recognized);
                    }
                    b"descrip" => {
                        let recognized = read_descrip_element(reader, value)?;
                        builder.descrip_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - term - term
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TermElement {
    pub content: String,
}

pub struct TermElementIterFunction;

impl iter::IterHelper<TermElement, TbxReaderError> for TermElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermElement>, TbxReaderError> {
        read_as_root_term_element(reader)
    }
}

pub fn iter_for_term_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TermElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_term_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"term" => {
                        break Ok(Some(read_term_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_term_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TermElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TermElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"term" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                builder.content(s_value.to_string());
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Element - termNote - termNote
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TermNoteElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<String>,
    pub content: String,
}

pub struct TermNoteElementIterFunction;

impl iter::IterHelper<TermNoteElement, TbxReaderError> for TermNoteElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermNoteElement>, TbxReaderError> {
        read_as_root_term_note_element(reader)
    }
}

pub fn iter_for_term_note_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TermNoteElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_term_note_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermNoteElement>, TbxReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termNote" => {
                        break Ok(Some(read_term_note_element(reader, start)?))
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => {break Ok(None)}
            _ => {}
        }
        buffer.clear();
    }
}

pub fn read_term_note_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TermNoteElement, TbxReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TermNoteElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_type_attribute(&attr)? {
                    builder.type_attribute(value);
                    continue;
                }
            }
            _ => {}
        }
    }
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"termNote" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                builder.content(s_value.to_string());
            }
            quick_xml::events::Event::Eof => {
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(builder.build()?)
}


// Attribute - type - TypeAttribute
pub fn read_type_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TbxReaderError>{
    if attr.key.local_name().as_ref() == b"type" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
}

// Attribute - style - StyleAttribute
#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum StyleAttribute {
    #[strum(serialize="dca")]
    Dca,
}

// Attribute - style - StyleAttribute
pub fn read_style_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<StyleAttribute>, TbxReaderError>{
    if attr.key.local_name().as_ref() == b"style" {
        let value = attr.unescape_value()?;
        let s = value.trim().to_lowercase();
        match s.parse(){
            Ok(value) => Ok(Some(value)),
            Err(error) => Err(TbxReaderError::AttributeStrumParserError("style", error, s)),
        }
    } else { Ok(None) }
}

// Attribute - lang - LangAttribute
#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum LangAttribute {
    #[strum(serialize="de")]
    De,
    #[strum(serialize="en")]
    En,
}

// Attribute - lang - LangAttribute
pub fn read_lang_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<LangAttribute>, TbxReaderError>{
    if attr.key.local_name().as_ref() == b"lang" {
        let value = attr.unescape_value()?;
        let s = value.trim().to_lowercase();
        match s.parse(){
            Ok(value) => Ok(Some(value)),
            Err(error) => Err(TbxReaderError::AttributeStrumParserError("lang", error, s)),
        }
    } else { Ok(None) }
}

// Attribute - xmlns - XmlnsAttribute
pub fn read_xmlns_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TbxReaderError>{
    if attr.key.local_name().as_ref() == b"xmlns" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
}

// Attribute - id - IdAttribute
pub fn read_id_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<u64>, TbxReaderError>{
    if attr.key.local_name().as_ref() == b"id" {
        let value = attr.unescape_value()?;
        Ok(Some(value.trim().to_lowercase().as_str().parse()?))
    } else { Ok(None) }
}


pub mod iter {
    pub trait IterHelper<I, E> {
        fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<I>, E>;
    }

    pub struct Iter<R, I, E, H> where H: IterHelper<I, E> {
        reader: quick_xml::reader::Reader<R>,
        _phantom: std::marker::PhantomData<fn(H, I) -> E>
    }

    impl<R, I, E, H> Iter<R, I, E, H>
    where
        H: IterHelper<I, E>
    {
        pub(super) fn new(reader: quick_xml::reader::Reader<R>) -> Self {
            Self { reader, _phantom: std::marker::PhantomData }
        }

        pub fn into_inner(self) -> quick_xml::reader::Reader<R> {
            self.reader
        }
    }

    impl<R, I, E, H> Iterator for Iter<R, I, E, H>
    where
        R: std::io::BufRead,
        E: std::error::Error,
        I: Sized,
        H: IterHelper<I, E>
    {
        type Item = Result<I, E>;

        fn next(&mut self) -> Option<Self::Item> {
            H::goto_next(&mut self.reader).transpose()
        }
    }

    use super::TbxReaderError;

    use super::TbxElement;
    use super::TbxElementIterFunction;
    /// Iterator for TbxElement
    pub type TbxElementIter<R> = Iter<R, TbxElement, TbxReaderError, TbxElementIterFunction>;

    use super::TbxHeaderElement;
    use super::TbxHeaderElementIterFunction;
    /// Iterator for TbxHeaderElement
    pub type TbxHeaderElementIter<R> = Iter<R, TbxHeaderElement, TbxReaderError, TbxHeaderElementIterFunction>;

    use super::FileDescElement;
    use super::FileDescElementIterFunction;
    /// Iterator for FileDescElement
    pub type FileDescElementIter<R> = Iter<R, FileDescElement, TbxReaderError, FileDescElementIterFunction>;

    use super::TitleStmtElement;
    use super::TitleStmtElementIterFunction;
    /// Iterator for TitleStmtElement
    pub type TitleStmtElementIter<R> = Iter<R, TitleStmtElement, TbxReaderError, TitleStmtElementIterFunction>;

    use super::TitleElement;
    use super::TitleElementIterFunction;
    /// Iterator for TitleElement
    pub type TitleElementIter<R> = Iter<R, TitleElement, TbxReaderError, TitleElementIterFunction>;

    use super::SourceDescElement;
    use super::SourceDescElementIterFunction;
    /// Iterator for SourceDescElement
    pub type SourceDescElementIter<R> = Iter<R, SourceDescElement, TbxReaderError, SourceDescElementIterFunction>;

    use super::PElement;
    use super::PElementIterFunction;
    /// Iterator for PElement
    pub type PElementIter<R> = Iter<R, PElement, TbxReaderError, PElementIterFunction>;

    use super::TextElement;
    use super::TextElementIterFunction;
    /// Iterator for TextElement
    pub type TextElementIter<R> = Iter<R, TextElement, TbxReaderError, TextElementIterFunction>;

    use super::BodyElement;
    use super::BodyElementIterFunction;
    /// Iterator for BodyElement
    pub type BodyElementIter<R> = Iter<R, BodyElement, TbxReaderError, BodyElementIterFunction>;

    use super::ConceptEntryElement;
    use super::ConceptEntryElementIterFunction;
    /// Iterator for ConceptEntryElement
    pub type ConceptEntryElementIter<R> = Iter<R, ConceptEntryElement, TbxReaderError, ConceptEntryElementIterFunction>;

    use super::DescripElement;
    use super::DescripElementIterFunction;
    /// Iterator for DescripElement
    pub type DescripElementIter<R> = Iter<R, DescripElement, TbxReaderError, DescripElementIterFunction>;

    use super::LangSecElement;
    use super::LangSecElementIterFunction;
    /// Iterator for LangSecElement
    pub type LangSecElementIter<R> = Iter<R, LangSecElement, TbxReaderError, LangSecElementIterFunction>;

    use super::TermSecElement;
    use super::TermSecElementIterFunction;
    /// Iterator for TermSecElement
    pub type TermSecElementIter<R> = Iter<R, TermSecElement, TbxReaderError, TermSecElementIterFunction>;

    use super::TermElement;
    use super::TermElementIterFunction;
    /// Iterator for TermElement
    pub type TermElementIter<R> = Iter<R, TermElement, TbxReaderError, TermElementIterFunction>;

    use super::TermNoteElement;
    use super::TermNoteElementIterFunction;
    /// Iterator for TermNoteElement
    pub type TermNoteElementIter<R> = Iter<R, TermNoteElement, TbxReaderError, TermNoteElementIterFunction>;
}

#[derive(Debug, thiserror::Error)]
pub enum TbxReaderError{
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
    #[error("Failed for \"{0}\" with {1} (parsed value: \"{2}\")")]
    AttributeStrumParserError(&'static str, strum::ParseError, String),
    #[error("Failed for \"{0}\" with {1} (parsed value: \"{2}\")")]
    ElementStrumParserError(&'static str, strum::ParseError, String),
    #[error(transparent)]
    TbxElementBuilderError(#[from] TbxElementBuilderError),
    #[error(transparent)]
    TbxHeaderElementBuilderError(#[from] TbxHeaderElementBuilderError),
    #[error(transparent)]
    FileDescElementBuilderError(#[from] FileDescElementBuilderError),
    #[error(transparent)]
    TitleStmtElementBuilderError(#[from] TitleStmtElementBuilderError),
    #[error(transparent)]
    TitleElementBuilderError(#[from] TitleElementBuilderError),
    #[error(transparent)]
    SourceDescElementBuilderError(#[from] SourceDescElementBuilderError),
    #[error(transparent)]
    PElementBuilderError(#[from] PElementBuilderError),
    #[error(transparent)]
    TextElementBuilderError(#[from] TextElementBuilderError),
    #[error(transparent)]
    BodyElementBuilderError(#[from] BodyElementBuilderError),
    #[error(transparent)]
    ConceptEntryElementBuilderError(#[from] ConceptEntryElementBuilderError),
    #[error(transparent)]
    DescripElementBuilderError(#[from] DescripElementBuilderError),
    #[error(transparent)]
    LangSecElementBuilderError(#[from] LangSecElementBuilderError),
    #[error(transparent)]
    TermSecElementBuilderError(#[from] TermSecElementBuilderError),
    #[error(transparent)]
    TermElementBuilderError(#[from] TermElementBuilderError),
    #[error(transparent)]
    TermNoteElementBuilderError(#[from] TermNoteElementBuilderError),
}