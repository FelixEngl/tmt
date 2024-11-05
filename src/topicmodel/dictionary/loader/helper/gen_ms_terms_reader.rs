//hash_signature:FmLi9BrcLOXQMqSqzt9Kbg==

/// Element - martif - e_martif
/// Encounters: 2
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct MartifElement {
    /// Meta Infos
    /// ```text
    /// Depth: 0
    ///     EnUs: 2
    ///```
    pub lang_attribute: LangAttribute,
    /// Meta Infos
    /// ```text
    /// Depth: 0
    ///     Tbx: 2
    ///```
    pub type_attribute: TypeAttribute,
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 1: - 1..1
    ///```
    #[builder(setter(custom))]
    pub martif_header_element: MartifHeaderElement,
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 1: - 1..1
    ///```
    #[builder(setter(custom))]
    pub text_element: TextElement,
}

impl MartifElementBuilder {
    pub fn martif_header_element(&mut self, value: MartifHeaderElement){
        assert!(self.martif_header_element.is_none(), "martif_header_element in MartifElement should be unset!");
        self.martif_header_element = Some(value);
    }
    pub fn text_element(&mut self, value: TextElement){
        assert!(self.text_element.is_none(), "text_element in MartifElement should be unset!");
        self.text_element = Some(value);
    }
}

pub struct MartifElementIterFunction;

impl iter::IterHelper<MartifElement, MartifReaderError> for MartifElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<MartifElement>, MartifReaderError> {
        read_as_root_martif_element(reader)
    }
}

pub fn iter_for_martif_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::MartifElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_martif_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<MartifElement>, MartifReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"martif" => {
                        break Ok(Some(read_martif_element(reader, start)?))
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

pub fn read_martif_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<MartifElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = MartifElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_lang_attribute(&attr)? {
                    builder.lang_attribute(value);
                    continue;
                }
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
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"martifHeader" => {
                        let recognized = read_martif_header_element(reader, start)?;
                        builder.martif_header_element(recognized);
                    }
                    b"text" => {
                        let recognized = read_text_element(reader, start)?;
                        builder.text_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"martif" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"martifHeader" => {
                        let recognized = read_martif_header_element(reader, value)?;
                        builder.martif_header_element(recognized);
                    }
                    b"text" => {
                        let recognized = read_text_element(reader, value)?;
                        builder.text_element(recognized);
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


/// Element - martifHeader - e_martifHeader
/// Encounters: 2
/// ```text
///     Depth 1: 1..1
///         - MartifElement (martif, e_martif): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct MartifHeaderElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 2: - 1..1
    ///```
    #[builder(setter(custom))]
    pub file_desc_element: FileDescElement,
}

impl MartifHeaderElementBuilder {
    pub fn file_desc_element(&mut self, value: FileDescElement){
        assert!(self.file_desc_element.is_none(), "file_desc_element in MartifHeaderElement should be unset!");
        self.file_desc_element = Some(value);
    }
}

pub struct MartifHeaderElementIterFunction;

impl iter::IterHelper<MartifHeaderElement, MartifReaderError> for MartifHeaderElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<MartifHeaderElement>, MartifReaderError> {
        read_as_root_martif_header_element(reader)
    }
}

pub fn iter_for_martif_header_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::MartifHeaderElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_martif_header_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<MartifHeaderElement>, MartifReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"martifHeader" => {
                        break Ok(Some(read_martif_header_element(reader, start)?))
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

pub fn read_martif_header_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<MartifHeaderElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = MartifHeaderElementBuilder::create_empty();
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
                    b"martifHeader" => {
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


/// Element - fileDesc - e_fileDesc
/// Encounters: 2
/// ```text
///     Depth 2: 1..1
///         - MartifHeaderElement (martifHeader, e_martifHeader): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct FileDescElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 3: - 1..1
    ///```
    #[builder(setter(custom))]
    pub title_stmt_element: TitleStmtElement,
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 3: - 1..1
    ///```
    #[builder(setter(custom))]
    pub source_desc_element: SourceDescElement,
}

impl FileDescElementBuilder {
    pub fn title_stmt_element(&mut self, value: TitleStmtElement){
        assert!(self.title_stmt_element.is_none(), "title_stmt_element in FileDescElement should be unset!");
        self.title_stmt_element = Some(value);
    }
    pub fn source_desc_element(&mut self, value: SourceDescElement){
        assert!(self.source_desc_element.is_none(), "source_desc_element in FileDescElement should be unset!");
        self.source_desc_element = Some(value);
    }
}

pub struct FileDescElementIterFunction;

impl iter::IterHelper<FileDescElement, MartifReaderError> for FileDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FileDescElement>, MartifReaderError> {
        read_as_root_file_desc_element(reader)
    }
}

pub fn iter_for_file_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::FileDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_file_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FileDescElement>, MartifReaderError>{
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

pub fn read_file_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<FileDescElement, MartifReaderError>{
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


/// Element - titleStmt - e_titleStmt
/// Encounters: 2
/// ```text
///     Depth 3: 1..1
///         - FileDescElement (fileDesc, e_fileDesc): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TitleStmtElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 4: - 1..1
    ///```
    #[builder(setter(custom))]
    pub title_element: TitleElement,
}

impl TitleStmtElementBuilder {
    pub fn title_element(&mut self, value: TitleElement){
        assert!(self.title_element.is_none(), "title_element in TitleStmtElement should be unset!");
        self.title_element = Some(value);
    }
}

pub struct TitleStmtElementIterFunction;

impl iter::IterHelper<TitleStmtElement, MartifReaderError> for TitleStmtElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleStmtElement>, MartifReaderError> {
        read_as_root_title_stmt_element(reader)
    }
}

pub fn iter_for_title_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TitleStmtElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_title_stmt_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleStmtElement>, MartifReaderError>{
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

pub fn read_title_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TitleStmtElement, MartifReaderError>{
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


/// Element - title - e_title
/// Encounters: 2
/// ```text
///     Depth 4: 1..1
///         - TitleStmtElement (titleStmt, e_titleStmt): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TitleElement {
    /// Content-Count: Overall=2 Unique=1 
    pub content: String,
}

pub struct TitleElementIterFunction;

impl iter::IterHelper<TitleElement, MartifReaderError> for TitleElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleElement>, MartifReaderError> {
        read_as_root_title_element(reader)
    }
}

pub fn iter_for_title_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TitleElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_title_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleElement>, MartifReaderError>{
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

pub fn read_title_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TitleElement, MartifReaderError>{
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
                let s_value = value.unescape()?;
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


/// Element - sourceDesc - e_sourceDesc
/// Encounters: 2
/// ```text
///     Depth 3: 1..1
///         - FileDescElement (fileDesc, e_fileDesc): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct SourceDescElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 4: - 1..1
    ///```
    #[builder(setter(custom))]
    pub p_element: PElement,
}

impl SourceDescElementBuilder {
    pub fn p_element(&mut self, value: PElement){
        assert!(self.p_element.is_none(), "p_element in SourceDescElement should be unset!");
        self.p_element = Some(value);
    }
}

pub struct SourceDescElementIterFunction;

impl iter::IterHelper<SourceDescElement, MartifReaderError> for SourceDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SourceDescElement>, MartifReaderError> {
        read_as_root_source_desc_element(reader)
    }
}

pub fn iter_for_source_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::SourceDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_source_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SourceDescElement>, MartifReaderError>{
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

pub fn read_source_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<SourceDescElement, MartifReaderError>{
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


/// Element - p - e_p
/// Encounters: 2
/// ```text
///     Depth 4: 1..1
///         - SourceDescElement (sourceDesc, e_sourceDesc): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PElement {
    /// Content-Count: Overall=2 Unique=1 
    pub content: String,
}

pub struct PElementIterFunction;

impl iter::IterHelper<PElement, MartifReaderError> for PElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PElement>, MartifReaderError> {
        read_as_root_p_element(reader)
    }
}

pub fn iter_for_p_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_p_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PElement>, MartifReaderError>{
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

pub fn read_p_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PElement, MartifReaderError>{
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
                let s_value = value.unescape()?;
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


/// Element - text - e_text
/// Encounters: 2
/// ```text
///     Depth 1: 1..1
///         - MartifElement (martif, e_martif): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TextElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 2
    ///        - Depth 2: - 1..1
    ///```
    #[builder(setter(custom))]
    pub body_element: BodyElement,
}

impl TextElementBuilder {
    pub fn body_element(&mut self, value: BodyElement){
        assert!(self.body_element.is_none(), "body_element in TextElement should be unset!");
        self.body_element = Some(value);
    }
}

pub struct TextElementIterFunction;

impl iter::IterHelper<TextElement, MartifReaderError> for TextElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TextElement>, MartifReaderError> {
        read_as_root_text_element(reader)
    }
}

pub fn iter_for_text_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TextElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_text_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TextElement>, MartifReaderError>{
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

pub fn read_text_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TextElement, MartifReaderError>{
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


/// Element - body - e_body
/// Encounters: 2
/// ```text
///     Depth 2: 1..1
///         - TextElement (text, e_text): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct BodyElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 40246
    ///        - Depth 3: - 4481..35765
    ///```
    #[builder(setter(custom), default)]
    pub term_entry_elements: Vec<TermEntryElement>,
}

impl BodyElementBuilder {
    pub fn term_entry_element(&mut self, value: TermEntryElement){
        let targ = self.term_entry_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct BodyElementIterFunction;

impl iter::IterHelper<BodyElement, MartifReaderError> for BodyElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<BodyElement>, MartifReaderError> {
        read_as_root_body_element(reader)
    }
}

pub fn iter_for_body_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::BodyElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_body_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<BodyElement>, MartifReaderError>{
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

pub fn read_body_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<BodyElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = BodyElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termEntry" => {
                        let recognized = read_term_entry_element(reader, start)?;
                        builder.term_entry_element(recognized);
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
                    b"termEntry" => {
                        let recognized = read_term_entry_element(reader, value)?;
                        builder.term_entry_element(recognized);
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


/// Element - termEntry - e_termEntry
/// Encounters: 40246
/// ```text
///     Depth 3: 4481..35765
///         - BodyElement (body, e_body): 4481..35765
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TermEntryElement {
    /// Meta Infos
    /// ```text
    /// Depth: 3
    ///     This value has 35927 different values.
    ///```
    pub id_attribute: String,
    ///Multiplicity:
    ///```text
    ///    Encounters: 80492
    ///        - Depth 4: - 2..2
    ///```
    #[builder(setter(custom), default)]
    pub lang_set_elements: Vec<LangSetElement>,
}

impl TermEntryElementBuilder {
    pub fn lang_set_element(&mut self, value: LangSetElement){
        let targ = self.lang_set_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct TermEntryElementIterFunction;

impl iter::IterHelper<TermEntryElement, MartifReaderError> for TermEntryElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermEntryElement>, MartifReaderError> {
        read_as_root_term_entry_element(reader)
    }
}

pub fn iter_for_term_entry_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TermEntryElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_term_entry_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermEntryElement>, MartifReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termEntry" => {
                        break Ok(Some(read_term_entry_element(reader, start)?))
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

pub fn read_term_entry_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TermEntryElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TermEntryElementBuilder::create_empty();
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
                    b"langSet" => {
                        let recognized = read_lang_set_element(reader, start)?;
                        builder.lang_set_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"termEntry" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"langSet" => {
                        let recognized = read_lang_set_element(reader, value)?;
                        builder.lang_set_element(recognized);
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


/// Element - langSet - e_langSet
/// Encounters: 80492
/// ```text
///     Depth 4: 2..2
///         - TermEntryElement (termEntry, e_termEntry): 2..2
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct LangSetElement {
    /// Meta Infos
    /// ```text
    /// Depth: 4
    ///     DeDe: 35765
    ///     EnGb: 4481
    ///     EnUs: 40246
    ///```
    pub lang_attribute: LangAttribute,
    ///Multiplicity:
    ///```text
    ///    Encounters: 40246
    ///        - Depth 5: - 0..1
    ///```
    #[builder(setter(custom), default)]
    pub descrip_grp_element: Option<DescripGrpElement>,
    ///Multiplicity:
    ///```text
    ///    Encounters: 81535
    ///        - Depth 5: - 1..7
    ///```
    #[builder(setter(custom), default)]
    pub ntig_elements: Vec<NtigElement>,
}

impl LangSetElementBuilder {
    pub fn descrip_grp_element(&mut self, value: DescripGrpElement){
        assert!(self.descrip_grp_element.is_none(), "descrip_grp_element in LangSetElement should be unset!");
        self.descrip_grp_element = Some(Some(value));
    }
    pub fn ntig_element(&mut self, value: NtigElement){
        let targ = self.ntig_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct LangSetElementIterFunction;

impl iter::IterHelper<LangSetElement, MartifReaderError> for LangSetElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<LangSetElement>, MartifReaderError> {
        read_as_root_lang_set_element(reader)
    }
}

pub fn iter_for_lang_set_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::LangSetElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_lang_set_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<LangSetElement>, MartifReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"langSet" => {
                        break Ok(Some(read_lang_set_element(reader, start)?))
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

pub fn read_lang_set_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<LangSetElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = LangSetElementBuilder::create_empty();
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
                    b"descripGrp" => {
                        let recognized = read_descrip_grp_element(reader, start)?;
                        builder.descrip_grp_element(recognized);
                    }
                    b"ntig" => {
                        let recognized = read_ntig_element(reader, start)?;
                        builder.ntig_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"langSet" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"descripGrp" => {
                        let recognized = read_descrip_grp_element(reader, value)?;
                        builder.descrip_grp_element(recognized);
                    }
                    b"ntig" => {
                        let recognized = read_ntig_element(reader, value)?;
                        builder.ntig_element(recognized);
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


/// Element - descripGrp - e_descripGrp
/// Encounters: 40246
/// ```text
///     Depth 5: 0..1
///         - LangSetElement (langSet, e_langSet): 0..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct DescripGrpElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 40246
    ///        - Depth 6: - 1..1
    ///```
    #[builder(setter(custom))]
    pub descrip_element: DescripElement,
}

impl DescripGrpElementBuilder {
    pub fn descrip_element(&mut self, value: DescripElement){
        assert!(self.descrip_element.is_none(), "descrip_element in DescripGrpElement should be unset!");
        self.descrip_element = Some(value);
    }
}

pub struct DescripGrpElementIterFunction;

impl iter::IterHelper<DescripGrpElement, MartifReaderError> for DescripGrpElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DescripGrpElement>, MartifReaderError> {
        read_as_root_descrip_grp_element(reader)
    }
}

pub fn iter_for_descrip_grp_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::DescripGrpElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_descrip_grp_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DescripGrpElement>, MartifReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"descripGrp" => {
                        break Ok(Some(read_descrip_grp_element(reader, start)?))
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

pub fn read_descrip_grp_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<DescripGrpElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = DescripGrpElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"descrip" => {
                        let recognized = read_descrip_element(reader, start)?;
                        builder.descrip_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"descripGrp" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
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


/// Element - descrip - e_descrip
/// Encounters: 40246
/// ```text
///     Depth 6: 1..1
///         - DescripGrpElement (descripGrp, e_descripGrp): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct DescripElement {
    /// Meta Infos
    /// ```text
    /// Depth: 6
    ///     Definition: 40246
    ///```
    pub type_attribute: TypeAttribute,
    /// Content-Count: Overall=40246 Unique=30510 
    pub content: String,
}

pub struct DescripElementIterFunction;

impl iter::IterHelper<DescripElement, MartifReaderError> for DescripElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DescripElement>, MartifReaderError> {
        read_as_root_descrip_element(reader)
    }
}

pub fn iter_for_descrip_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::DescripElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_descrip_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DescripElement>, MartifReaderError>{
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

pub fn read_descrip_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<DescripElement, MartifReaderError>{
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
                let s_value = value.unescape()?;
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


/// Element - ntig - e_ntig
/// Encounters: 81535
/// ```text
///     Depth 5: 1..7
///         - LangSetElement (langSet, e_langSet): 1..7
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct NtigElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 81535
    ///        - Depth 6: - 1..1
    ///```
    #[builder(setter(custom))]
    pub term_grp_element: TermGrpElement,
}

impl NtigElementBuilder {
    pub fn term_grp_element(&mut self, value: TermGrpElement){
        assert!(self.term_grp_element.is_none(), "term_grp_element in NtigElement should be unset!");
        self.term_grp_element = Some(value);
    }
}

pub struct NtigElementIterFunction;

impl iter::IterHelper<NtigElement, MartifReaderError> for NtigElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NtigElement>, MartifReaderError> {
        read_as_root_ntig_element(reader)
    }
}

pub fn iter_for_ntig_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::NtigElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_ntig_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NtigElement>, MartifReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"ntig" => {
                        break Ok(Some(read_ntig_element(reader, start)?))
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

pub fn read_ntig_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<NtigElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = NtigElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termGrp" => {
                        let recognized = read_term_grp_element(reader, start)?;
                        builder.term_grp_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"ntig" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"termGrp" => {
                        let recognized = read_term_grp_element(reader, value)?;
                        builder.term_grp_element(recognized);
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


/// Element - termGrp - e_termGrp
/// Encounters: 81535
/// ```text
///     Depth 6: 1..1
///         - NtigElement (ntig, e_ntig): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TermGrpElement {
    ///Multiplicity:
    ///```text
    ///    Encounters: 81535
    ///        - Depth 7: - 1..1
    ///```
    #[builder(setter(custom))]
    pub term_element: TermElement,
    ///Multiplicity:
    ///```text
    ///    Encounters: 81535
    ///        - Depth 7: - 1..1
    ///```
    #[builder(setter(custom))]
    pub term_note_element: TermNoteElement,
}

impl TermGrpElementBuilder {
    pub fn term_element(&mut self, value: TermElement){
        assert!(self.term_element.is_none(), "term_element in TermGrpElement should be unset!");
        self.term_element = Some(value);
    }
    pub fn term_note_element(&mut self, value: TermNoteElement){
        assert!(self.term_note_element.is_none(), "term_note_element in TermGrpElement should be unset!");
        self.term_note_element = Some(value);
    }
}

pub struct TermGrpElementIterFunction;

impl iter::IterHelper<TermGrpElement, MartifReaderError> for TermGrpElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermGrpElement>, MartifReaderError> {
        read_as_root_term_grp_element(reader)
    }
}

pub fn iter_for_term_grp_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TermGrpElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_term_grp_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermGrpElement>, MartifReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"termGrp" => {
                        break Ok(Some(read_term_grp_element(reader, start)?))
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

pub fn read_term_grp_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TermGrpElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TermGrpElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"term" => {
                        let recognized = read_term_element(reader, start)?;
                        builder.term_element(recognized);
                    }
                    b"termNote" => {
                        let recognized = read_term_note_element(reader, start)?;
                        builder.term_note_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"termGrp" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"term" => {
                        let recognized = read_term_element(reader, value)?;
                        builder.term_element(recognized);
                    }
                    b"termNote" => {
                        let recognized = read_term_note_element(reader, value)?;
                        builder.term_note_element(recognized);
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


/// Element - term - e_term
/// Encounters: 81535
/// ```text
///     Depth 7: 1..1
///         - TermGrpElement (termGrp, e_termGrp): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TermElement {
    /// Meta Infos
    /// ```text
    /// Depth: 7
    ///     This value has 76231 different values.
    ///```
    pub id_attribute: String,
    /// Content-Count: Overall=81535 Unique=59360 
    pub content: String,
}

pub struct TermElementIterFunction;

impl iter::IterHelper<TermElement, MartifReaderError> for TermElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermElement>, MartifReaderError> {
        read_as_root_term_element(reader)
    }
}

pub fn iter_for_term_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TermElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_term_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermElement>, MartifReaderError>{
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

pub fn read_term_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TermElement, MartifReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TermElementBuilder::create_empty();
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
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"term" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = value.unescape()?;
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


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, strum::Display, strum::EnumString)]
pub enum ETermNoteElement {
    #[strum(serialize="Noun")]
    Noun,
    #[strum(serialize="Other")]
    Other,
    #[strum(serialize="Verb")]
    Verb,
    #[strum(serialize="Proper Noun")]
    ProperNoun,
    #[strum(serialize="Adjective")]
    Adjective,
    #[strum(serialize="Adverb")]
    Adverb,
}

/// Element - termNote - e_termNote
/// Encounters: 81535
/// ```text
///     Depth 7: 1..1
///         - TermGrpElement (termGrp, e_termGrp): 1..1
/// ```
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TermNoteElement {
    /// Meta Infos
    /// ```text
    /// Depth: 7
    ///     PartOfSpeech: 81535
    ///```
    pub type_attribute: TypeAttribute,
    /// Content-Count: Overall=81535 Unique=6 
    pub content: ETermNoteElement,
}

pub struct TermNoteElementIterFunction;

impl iter::IterHelper<TermNoteElement, MartifReaderError> for TermNoteElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermNoteElement>, MartifReaderError> {
        read_as_root_term_note_element(reader)
    }
}

pub fn iter_for_term_note_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TermNoteElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_term_note_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TermNoteElement>, MartifReaderError>{
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

pub fn read_term_note_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TermNoteElement, MartifReaderError>{
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
                let s_value = value.unescape()?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(MartifReaderError::ElementStrumParserError("termNote", error, s.to_string()));
                    }
                }
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


// Attribute - type - a_type - TypeAttribute
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, strum::Display, strum::EnumString)]
pub enum TypeAttribute {
    #[strum(serialize="partofspeech")]
    PartOfSpeech,
    #[strum(serialize="definition")]
    Definition,
    #[strum(serialize="tbx")]
    Tbx,
}

/// Attribute - type - a_type
pub fn read_type_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<TypeAttribute>, MartifReaderError>{
    if attr.key.local_name().as_ref() == b"type" {
        let value = attr.unescape_value()?;
        let s = value.trim().to_lowercase();
        match s.parse(){
            Ok(value) => Ok(Some(value)),
            Err(error) => Err(MartifReaderError::AttributeStrumParserError("type", error, s)),
        }
    } else { Ok(None) }
}

// Attribute - lang - a_lang - LangAttribute
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, strum::Display, strum::EnumString)]
pub enum LangAttribute {
    #[strum(serialize="en-us")]
    EnUs,
    #[strum(serialize="de-de")]
    DeDe,
    #[strum(serialize="en-gb")]
    EnGb,
}

/// Attribute - lang - a_lang
pub fn read_lang_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<LangAttribute>, MartifReaderError>{
    if attr.key.local_name().as_ref() == b"lang" {
        let value = attr.unescape_value()?;
        let s = value.trim().to_lowercase();
        match s.parse(){
            Ok(value) => Ok(Some(value)),
            Err(error) => Err(MartifReaderError::AttributeStrumParserError("lang", error, s)),
        }
    } else { Ok(None) }
}

/// Attribute - id - a_id
/// Has 112158 unique values.
pub fn read_id_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, MartifReaderError>{
    if attr.key.local_name().as_ref() == b"id" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))
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

    use super::MartifReaderError;

    use super::MartifElement;
    use super::MartifElementIterFunction;
    /// Iterator for MartifElement
    pub type MartifElementIter<R> = Iter<R, MartifElement, MartifReaderError, MartifElementIterFunction>;

    use super::MartifHeaderElement;
    use super::MartifHeaderElementIterFunction;
    /// Iterator for MartifHeaderElement
    pub type MartifHeaderElementIter<R> = Iter<R, MartifHeaderElement, MartifReaderError, MartifHeaderElementIterFunction>;

    use super::FileDescElement;
    use super::FileDescElementIterFunction;
    /// Iterator for FileDescElement
    pub type FileDescElementIter<R> = Iter<R, FileDescElement, MartifReaderError, FileDescElementIterFunction>;

    use super::TitleStmtElement;
    use super::TitleStmtElementIterFunction;
    /// Iterator for TitleStmtElement
    pub type TitleStmtElementIter<R> = Iter<R, TitleStmtElement, MartifReaderError, TitleStmtElementIterFunction>;

    use super::TitleElement;
    use super::TitleElementIterFunction;
    /// Iterator for TitleElement
    pub type TitleElementIter<R> = Iter<R, TitleElement, MartifReaderError, TitleElementIterFunction>;

    use super::SourceDescElement;
    use super::SourceDescElementIterFunction;
    /// Iterator for SourceDescElement
    pub type SourceDescElementIter<R> = Iter<R, SourceDescElement, MartifReaderError, SourceDescElementIterFunction>;

    use super::PElement;
    use super::PElementIterFunction;
    /// Iterator for PElement
    pub type PElementIter<R> = Iter<R, PElement, MartifReaderError, PElementIterFunction>;

    use super::TextElement;
    use super::TextElementIterFunction;
    /// Iterator for TextElement
    pub type TextElementIter<R> = Iter<R, TextElement, MartifReaderError, TextElementIterFunction>;

    use super::BodyElement;
    use super::BodyElementIterFunction;
    /// Iterator for BodyElement
    pub type BodyElementIter<R> = Iter<R, BodyElement, MartifReaderError, BodyElementIterFunction>;

    use super::TermEntryElement;
    use super::TermEntryElementIterFunction;
    /// Iterator for TermEntryElement
    pub type TermEntryElementIter<R> = Iter<R, TermEntryElement, MartifReaderError, TermEntryElementIterFunction>;

    use super::LangSetElement;
    use super::LangSetElementIterFunction;
    /// Iterator for LangSetElement
    pub type LangSetElementIter<R> = Iter<R, LangSetElement, MartifReaderError, LangSetElementIterFunction>;

    use super::DescripGrpElement;
    use super::DescripGrpElementIterFunction;
    /// Iterator for DescripGrpElement
    pub type DescripGrpElementIter<R> = Iter<R, DescripGrpElement, MartifReaderError, DescripGrpElementIterFunction>;

    use super::DescripElement;
    use super::DescripElementIterFunction;
    /// Iterator for DescripElement
    pub type DescripElementIter<R> = Iter<R, DescripElement, MartifReaderError, DescripElementIterFunction>;

    use super::NtigElement;
    use super::NtigElementIterFunction;
    /// Iterator for NtigElement
    pub type NtigElementIter<R> = Iter<R, NtigElement, MartifReaderError, NtigElementIterFunction>;

    use super::TermGrpElement;
    use super::TermGrpElementIterFunction;
    /// Iterator for TermGrpElement
    pub type TermGrpElementIter<R> = Iter<R, TermGrpElement, MartifReaderError, TermGrpElementIterFunction>;

    use super::TermElement;
    use super::TermElementIterFunction;
    /// Iterator for TermElement
    pub type TermElementIter<R> = Iter<R, TermElement, MartifReaderError, TermElementIterFunction>;

    use super::TermNoteElement;
    use super::TermNoteElementIterFunction;
    /// Iterator for TermNoteElement
    pub type TermNoteElementIter<R> = Iter<R, TermNoteElement, MartifReaderError, TermNoteElementIterFunction>;
}

#[derive(Debug, thiserror::Error)]
pub enum MartifReaderError{
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
    MartifElementBuilderError(#[from] MartifElementBuilderError),
    #[error(transparent)]
    MartifHeaderElementBuilderError(#[from] MartifHeaderElementBuilderError),
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
    TermEntryElementBuilderError(#[from] TermEntryElementBuilderError),
    #[error(transparent)]
    LangSetElementBuilderError(#[from] LangSetElementBuilderError),
    #[error(transparent)]
    DescripGrpElementBuilderError(#[from] DescripGrpElementBuilderError),
    #[error(transparent)]
    DescripElementBuilderError(#[from] DescripElementBuilderError),
    #[error(transparent)]
    NtigElementBuilderError(#[from] NtigElementBuilderError),
    #[error(transparent)]
    TermGrpElementBuilderError(#[from] TermGrpElementBuilderError),
    #[error(transparent)]
    TermElementBuilderError(#[from] TermElementBuilderError),
    #[error(transparent)]
    TermNoteElementBuilderError(#[from] TermNoteElementBuilderError),
}