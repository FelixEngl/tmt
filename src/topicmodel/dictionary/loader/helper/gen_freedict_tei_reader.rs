//hash_signature:rSKJHQCgZ/3zh1K35Vk3bQ==

// Element - TEI - TEI
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TeiElement {
    pub xmlns_attribute: String,
    pub version_attribute: f64,
    #[builder(setter(custom), default)]
    pub tei_header_element: Option<TeiHeaderElement>,
    #[builder(setter(custom), default)]
    pub text_element: Option<TextElement>,
}

impl TeiElementBuilder {
    pub fn tei_header_element(&mut self, value: TeiHeaderElement){
        assert!(self.tei_header_element.is_none(), "tei_header_element in TeiElement should be unset!");
        self.tei_header_element = Some(Some(value));
    }
    pub fn text_element(&mut self, value: TextElement){
        assert!(self.text_element.is_none(), "text_element in TeiElement should be unset!");
        self.text_element = Some(Some(value));
    }
}

pub struct TeiElementIterFunction;

impl iter::IterHelper<TeiElement, TeiReaderError> for TeiElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TeiElement>, TeiReaderError> {
        read_as_root_tei_element(reader)
    }
}

pub fn iter_for_tei_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TeiElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_tei_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TeiElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"TEI" => {
                        break Ok(Some(read_tei_element(reader, start)?))
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

pub fn read_tei_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TeiElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TeiElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_xmlns_attribute(&attr)? {
                    builder.xmlns_attribute(value);
                    continue;
                }
                if let Some(value) = read_version_attribute(&attr)? {
                    builder.version_attribute(value);
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
                    b"teiHeader" => {
                        let recognized = read_tei_header_element(reader, start)?;
                        builder.tei_header_element(recognized);
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
                    b"TEI" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"teiHeader" => {
                        let recognized = read_tei_header_element(reader, value)?;
                        builder.tei_header_element(recognized);
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


// Element - teiHeader - teiHeader
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TeiHeaderElement {
    #[builder(setter(strip_option), default)]
    pub lang_attribute: Option<LangAttribute>,
    #[builder(setter(custom), default)]
    pub revision_desc_element: Option<RevisionDescElement>,
    #[builder(setter(custom), default)]
    pub file_desc_element: Option<FileDescElement>,
    #[builder(setter(custom), default)]
    pub encoding_desc_element: Option<EncodingDescElement>,
}

impl TeiHeaderElementBuilder {
    pub fn revision_desc_element(&mut self, value: RevisionDescElement){
        assert!(self.revision_desc_element.is_none(), "revision_desc_element in TeiHeaderElement should be unset!");
        self.revision_desc_element = Some(Some(value));
    }
    pub fn file_desc_element(&mut self, value: FileDescElement){
        assert!(self.file_desc_element.is_none(), "file_desc_element in TeiHeaderElement should be unset!");
        self.file_desc_element = Some(Some(value));
    }
    pub fn encoding_desc_element(&mut self, value: EncodingDescElement){
        assert!(self.encoding_desc_element.is_none(), "encoding_desc_element in TeiHeaderElement should be unset!");
        self.encoding_desc_element = Some(Some(value));
    }
}

pub struct TeiHeaderElementIterFunction;

impl iter::IterHelper<TeiHeaderElement, TeiReaderError> for TeiHeaderElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TeiHeaderElement>, TeiReaderError> {
        read_as_root_tei_header_element(reader)
    }
}

pub fn iter_for_tei_header_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TeiHeaderElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_tei_header_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TeiHeaderElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"teiHeader" => {
                        break Ok(Some(read_tei_header_element(reader, start)?))
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

pub fn read_tei_header_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TeiHeaderElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TeiHeaderElementBuilder::create_empty();
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
                    b"revisionDesc" => {
                        let recognized = read_revision_desc_element(reader, start)?;
                        builder.revision_desc_element(recognized);
                    }
                    b"fileDesc" => {
                        let recognized = read_file_desc_element(reader, start)?;
                        builder.file_desc_element(recognized);
                    }
                    b"encodingDesc" => {
                        let recognized = read_encoding_desc_element(reader, start)?;
                        builder.encoding_desc_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"teiHeader" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"revisionDesc" => {
                        let recognized = read_revision_desc_element(reader, value)?;
                        builder.revision_desc_element(recognized);
                    }
                    b"fileDesc" => {
                        let recognized = read_file_desc_element(reader, value)?;
                        builder.file_desc_element(recognized);
                    }
                    b"encodingDesc" => {
                        let recognized = read_encoding_desc_element(reader, value)?;
                        builder.encoding_desc_element(recognized);
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
    #[builder(setter(custom), default)]
    pub title_stmt_element: Option<TitleStmtElement>,
    #[builder(setter(custom), default)]
    pub extent_element: Option<ExtentElement>,
    #[builder(setter(custom), default)]
    pub edition_stmt_element: Option<EditionStmtElement>,
    #[builder(setter(custom), default)]
    pub publication_stmt_element: Option<PublicationStmtElement>,
    #[builder(setter(custom), default)]
    pub notes_stmt_element: Option<NotesStmtElement>,
    #[builder(setter(custom), default)]
    pub source_desc_element: Option<SourceDescElement>,
}

impl FileDescElementBuilder {
    pub fn title_stmt_element(&mut self, value: TitleStmtElement){
        assert!(self.title_stmt_element.is_none(), "title_stmt_element in FileDescElement should be unset!");
        self.title_stmt_element = Some(Some(value));
    }
    pub fn extent_element(&mut self, value: ExtentElement){
        assert!(self.extent_element.is_none(), "extent_element in FileDescElement should be unset!");
        self.extent_element = Some(Some(value));
    }
    pub fn edition_stmt_element(&mut self, value: EditionStmtElement){
        assert!(self.edition_stmt_element.is_none(), "edition_stmt_element in FileDescElement should be unset!");
        self.edition_stmt_element = Some(Some(value));
    }
    pub fn publication_stmt_element(&mut self, value: PublicationStmtElement){
        assert!(self.publication_stmt_element.is_none(), "publication_stmt_element in FileDescElement should be unset!");
        self.publication_stmt_element = Some(Some(value));
    }
    pub fn notes_stmt_element(&mut self, value: NotesStmtElement){
        assert!(self.notes_stmt_element.is_none(), "notes_stmt_element in FileDescElement should be unset!");
        self.notes_stmt_element = Some(Some(value));
    }
    pub fn source_desc_element(&mut self, value: SourceDescElement){
        assert!(self.source_desc_element.is_none(), "source_desc_element in FileDescElement should be unset!");
        self.source_desc_element = Some(Some(value));
    }
}

pub struct FileDescElementIterFunction;

impl iter::IterHelper<FileDescElement, TeiReaderError> for FileDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FileDescElement>, TeiReaderError> {
        read_as_root_file_desc_element(reader)
    }
}

pub fn iter_for_file_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::FileDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_file_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FileDescElement>, TeiReaderError>{
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

pub fn read_file_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<FileDescElement, TeiReaderError>{
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
                    b"extent" => {
                        let recognized = read_extent_element(reader, start)?;
                        builder.extent_element(recognized);
                    }
                    b"editionStmt" => {
                        let recognized = read_edition_stmt_element(reader, start)?;
                        builder.edition_stmt_element(recognized);
                    }
                    b"publicationStmt" => {
                        let recognized = read_publication_stmt_element(reader, start)?;
                        builder.publication_stmt_element(recognized);
                    }
                    b"notesStmt" => {
                        let recognized = read_notes_stmt_element(reader, start)?;
                        builder.notes_stmt_element(recognized);
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
                    b"extent" => {
                        let recognized = read_extent_element(reader, value)?;
                        builder.extent_element(recognized);
                    }
                    b"editionStmt" => {
                        let recognized = read_edition_stmt_element(reader, value)?;
                        builder.edition_stmt_element(recognized);
                    }
                    b"publicationStmt" => {
                        let recognized = read_publication_stmt_element(reader, value)?;
                        builder.publication_stmt_element(recognized);
                    }
                    b"notesStmt" => {
                        let recognized = read_notes_stmt_element(reader, value)?;
                        builder.notes_stmt_element(recognized);
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
    #[builder(setter(custom), default)]
    pub author_element: Option<AuthorElement>,
    #[builder(setter(custom), default)]
    pub resp_stmt_element: Option<RespStmtElement>,
    #[builder(setter(custom), default)]
    pub title_element: Option<TitleElement>,
    #[builder(setter(custom), default)]
    pub editor_element: Option<EditorElement>,
}

impl TitleStmtElementBuilder {
    pub fn author_element(&mut self, value: AuthorElement){
        assert!(self.author_element.is_none(), "author_element in TitleStmtElement should be unset!");
        self.author_element = Some(Some(value));
    }
    pub fn resp_stmt_element(&mut self, value: RespStmtElement){
        assert!(self.resp_stmt_element.is_none(), "resp_stmt_element in TitleStmtElement should be unset!");
        self.resp_stmt_element = Some(Some(value));
    }
    pub fn title_element(&mut self, value: TitleElement){
        assert!(self.title_element.is_none(), "title_element in TitleStmtElement should be unset!");
        self.title_element = Some(Some(value));
    }
    pub fn editor_element(&mut self, value: EditorElement){
        assert!(self.editor_element.is_none(), "editor_element in TitleStmtElement should be unset!");
        self.editor_element = Some(Some(value));
    }
}

pub struct TitleStmtElementIterFunction;

impl iter::IterHelper<TitleStmtElement, TeiReaderError> for TitleStmtElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleStmtElement>, TeiReaderError> {
        read_as_root_title_stmt_element(reader)
    }
}

pub fn iter_for_title_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TitleStmtElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_title_stmt_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleStmtElement>, TeiReaderError>{
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

pub fn read_title_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TitleStmtElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TitleStmtElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"author" => {
                        let recognized = read_author_element(reader, start)?;
                        builder.author_element(recognized);
                    }
                    b"respStmt" => {
                        let recognized = read_resp_stmt_element(reader, start)?;
                        builder.resp_stmt_element(recognized);
                    }
                    b"title" => {
                        let recognized = read_title_element(reader, start)?;
                        builder.title_element(recognized);
                    }
                    b"editor" => {
                        let recognized = read_editor_element(reader, start)?;
                        builder.editor_element(recognized);
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
                    b"author" => {
                        let recognized = read_author_element(reader, value)?;
                        builder.author_element(recognized);
                    }
                    b"respStmt" => {
                        let recognized = read_resp_stmt_element(reader, value)?;
                        builder.resp_stmt_element(recognized);
                    }
                    b"title" => {
                        let recognized = read_title_element(reader, value)?;
                        builder.title_element(recognized);
                    }
                    b"editor" => {
                        let recognized = read_editor_element(reader, value)?;
                        builder.editor_element(recognized);
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

impl iter::IterHelper<TitleElement, TeiReaderError> for TitleElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleElement>, TeiReaderError> {
        read_as_root_title_element(reader)
    }
}

pub fn iter_for_title_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TitleElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_title_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TitleElement>, TeiReaderError>{
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

pub fn read_title_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TitleElement, TeiReaderError>{
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


// Element - author - author
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct AuthorElement {
    pub content: String,
}

pub struct AuthorElementIterFunction;

impl iter::IterHelper<AuthorElement, TeiReaderError> for AuthorElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<AuthorElement>, TeiReaderError> {
        read_as_root_author_element(reader)
    }
}

pub fn iter_for_author_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::AuthorElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_author_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<AuthorElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"author" => {
                        break Ok(Some(read_author_element(reader, start)?))
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

pub fn read_author_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<AuthorElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = AuthorElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"author" => {
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


// Element - editor - editor
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct EditorElement {
    pub content: String,
}

pub struct EditorElementIterFunction;

impl iter::IterHelper<EditorElement, TeiReaderError> for EditorElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EditorElement>, TeiReaderError> {
        read_as_root_editor_element(reader)
    }
}

pub fn iter_for_editor_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::EditorElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_editor_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EditorElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"editor" => {
                        break Ok(Some(read_editor_element(reader, start)?))
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

pub fn read_editor_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<EditorElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = EditorElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"editor" => {
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


// Element - respStmt - respStmt
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct RespStmtElement {
    #[builder(setter(custom), default)]
    pub name_element: Option<NameElement>,
    #[builder(setter(custom), default)]
    pub resp_element: Option<RespElement>,
}

impl RespStmtElementBuilder {
    pub fn name_element(&mut self, value: NameElement){
        assert!(self.name_element.is_none(), "name_element in RespStmtElement should be unset!");
        self.name_element = Some(Some(value));
    }
    pub fn resp_element(&mut self, value: RespElement){
        assert!(self.resp_element.is_none(), "resp_element in RespStmtElement should be unset!");
        self.resp_element = Some(Some(value));
    }
}

pub struct RespStmtElementIterFunction;

impl iter::IterHelper<RespStmtElement, TeiReaderError> for RespStmtElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RespStmtElement>, TeiReaderError> {
        read_as_root_resp_stmt_element(reader)
    }
}

pub fn iter_for_resp_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::RespStmtElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_resp_stmt_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RespStmtElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"respStmt" => {
                        break Ok(Some(read_resp_stmt_element(reader, start)?))
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

pub fn read_resp_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<RespStmtElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = RespStmtElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"name" => {
                        let recognized = read_name_element(reader, start)?;
                        builder.name_element(recognized);
                    }
                    b"resp" => {
                        let recognized = read_resp_element(reader, start)?;
                        builder.resp_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"respStmt" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"name" => {
                        let recognized = read_name_element(reader, value)?;
                        builder.name_element(recognized);
                    }
                    b"resp" => {
                        let recognized = read_resp_element(reader, value)?;
                        builder.resp_element(recognized);
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum ERespElement {
    #[strum(serialize="Maintainer")]
    Maintainer,
}

// Element - resp - resp
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct RespElement {
    pub content: ERespElement,
}

pub struct RespElementIterFunction;

impl iter::IterHelper<RespElement, TeiReaderError> for RespElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RespElement>, TeiReaderError> {
        read_as_root_resp_element(reader)
    }
}

pub fn iter_for_resp_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::RespElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_resp_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RespElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"resp" => {
                        break Ok(Some(read_resp_element(reader, start)?))
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

pub fn read_resp_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<RespElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = RespElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"resp" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(TeiReaderError::ElementStrumParserError("resp", error, s.to_string()));
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


// Element - name - name
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct NameElement {
    pub content: String,
}

pub struct NameElementIterFunction;

impl iter::IterHelper<NameElement, TeiReaderError> for NameElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NameElement>, TeiReaderError> {
        read_as_root_name_element(reader)
    }
}

pub fn iter_for_name_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::NameElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_name_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NameElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"name" => {
                        break Ok(Some(read_name_element(reader, start)?))
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

pub fn read_name_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<NameElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = NameElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"name" => {
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


// Element - editionStmt - editionStmt
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct EditionStmtElement {
    #[builder(setter(custom), default)]
    pub edition_element: Option<EditionElement>,
}

impl EditionStmtElementBuilder {
    pub fn edition_element(&mut self, value: EditionElement){
        assert!(self.edition_element.is_none(), "edition_element in EditionStmtElement should be unset!");
        self.edition_element = Some(Some(value));
    }
}

pub struct EditionStmtElementIterFunction;

impl iter::IterHelper<EditionStmtElement, TeiReaderError> for EditionStmtElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EditionStmtElement>, TeiReaderError> {
        read_as_root_edition_stmt_element(reader)
    }
}

pub fn iter_for_edition_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::EditionStmtElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_edition_stmt_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EditionStmtElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"editionStmt" => {
                        break Ok(Some(read_edition_stmt_element(reader, start)?))
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

pub fn read_edition_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<EditionStmtElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = EditionStmtElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"edition" => {
                        let recognized = read_edition_element(reader, start)?;
                        builder.edition_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"editionStmt" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"edition" => {
                        let recognized = read_edition_element(reader, value)?;
                        builder.edition_element(recognized);
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


// Element - edition - edition
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct EditionElement {
    pub content: String,
}

pub struct EditionElementIterFunction;

impl iter::IterHelper<EditionElement, TeiReaderError> for EditionElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EditionElement>, TeiReaderError> {
        read_as_root_edition_element(reader)
    }
}

pub fn iter_for_edition_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::EditionElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_edition_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EditionElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"edition" => {
                        break Ok(Some(read_edition_element(reader, start)?))
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

pub fn read_edition_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<EditionElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = EditionElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"edition" => {
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


// Element - extent - extent
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct ExtentElement {
    pub content: String,
}

pub struct ExtentElementIterFunction;

impl iter::IterHelper<ExtentElement, TeiReaderError> for ExtentElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ExtentElement>, TeiReaderError> {
        read_as_root_extent_element(reader)
    }
}

pub fn iter_for_extent_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::ExtentElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_extent_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ExtentElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"extent" => {
                        break Ok(Some(read_extent_element(reader, start)?))
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

pub fn read_extent_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<ExtentElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = ExtentElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"extent" => {
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


// Element - publicationStmt - publicationStmt
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PublicationStmtElement {
    #[builder(setter(custom), default)]
    pub publisher_element: Option<PublisherElement>,
    #[builder(setter(custom), default)]
    pub availability_element: Option<AvailabilityElement>,
    #[builder(setter(custom), default)]
    pub pub_place_element: Option<PubPlaceElement>,
    #[builder(setter(custom), default)]
    pub date_element: Option<DateElement>,
}

impl PublicationStmtElementBuilder {
    pub fn publisher_element(&mut self, value: PublisherElement){
        assert!(self.publisher_element.is_none(), "publisher_element in PublicationStmtElement should be unset!");
        self.publisher_element = Some(Some(value));
    }
    pub fn availability_element(&mut self, value: AvailabilityElement){
        assert!(self.availability_element.is_none(), "availability_element in PublicationStmtElement should be unset!");
        self.availability_element = Some(Some(value));
    }
    pub fn pub_place_element(&mut self, value: PubPlaceElement){
        assert!(self.pub_place_element.is_none(), "pub_place_element in PublicationStmtElement should be unset!");
        self.pub_place_element = Some(Some(value));
    }
    pub fn date_element(&mut self, value: DateElement){
        assert!(self.date_element.is_none(), "date_element in PublicationStmtElement should be unset!");
        self.date_element = Some(Some(value));
    }
}

pub struct PublicationStmtElementIterFunction;

impl iter::IterHelper<PublicationStmtElement, TeiReaderError> for PublicationStmtElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PublicationStmtElement>, TeiReaderError> {
        read_as_root_publication_stmt_element(reader)
    }
}

pub fn iter_for_publication_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PublicationStmtElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_publication_stmt_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PublicationStmtElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"publicationStmt" => {
                        break Ok(Some(read_publication_stmt_element(reader, start)?))
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

pub fn read_publication_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PublicationStmtElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PublicationStmtElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"publisher" => {
                        let recognized = read_publisher_element(reader, start)?;
                        builder.publisher_element(recognized);
                    }
                    b"availability" => {
                        let recognized = read_availability_element(reader, start)?;
                        builder.availability_element(recognized);
                    }
                    b"pubPlace" => {
                        let recognized = read_pub_place_element(reader, start)?;
                        builder.pub_place_element(recognized);
                    }
                    b"date" => {
                        let recognized = read_date_element(reader, start)?;
                        builder.date_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"publicationStmt" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"publisher" => {
                        let recognized = read_publisher_element(reader, value)?;
                        builder.publisher_element(recognized);
                    }
                    b"availability" => {
                        let recognized = read_availability_element(reader, value)?;
                        builder.availability_element(recognized);
                    }
                    b"pubPlace" => {
                        let recognized = read_pub_place_element(reader, value)?;
                        builder.pub_place_element(recognized);
                    }
                    b"date" => {
                        let recognized = read_date_element(reader, value)?;
                        builder.date_element(recognized);
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum EPublisherElement {
    #[strum(serialize="FreeDict")]
    FreeDict,
}

// Element - publisher - publisher
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PublisherElement {
    pub content: EPublisherElement,
}

pub struct PublisherElementIterFunction;

impl iter::IterHelper<PublisherElement, TeiReaderError> for PublisherElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PublisherElement>, TeiReaderError> {
        read_as_root_publisher_element(reader)
    }
}

pub fn iter_for_publisher_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PublisherElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_publisher_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PublisherElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"publisher" => {
                        break Ok(Some(read_publisher_element(reader, start)?))
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

pub fn read_publisher_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PublisherElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PublisherElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"publisher" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(TeiReaderError::ElementStrumParserError("publisher", error, s.to_string()));
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


// Element - availability - availability
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct AvailabilityElement {
    pub status_attribute: StatusAttribute,
    #[builder(setter(custom), default)]
    pub p_elements: Vec<PElement>,
}

impl AvailabilityElementBuilder {
    pub fn p_element(&mut self, value: PElement){
        let targ = self.p_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct AvailabilityElementIterFunction;

impl iter::IterHelper<AvailabilityElement, TeiReaderError> for AvailabilityElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<AvailabilityElement>, TeiReaderError> {
        read_as_root_availability_element(reader)
    }
}

pub fn iter_for_availability_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::AvailabilityElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_availability_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<AvailabilityElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"availability" => {
                        break Ok(Some(read_availability_element(reader, start)?))
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

pub fn read_availability_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<AvailabilityElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = AvailabilityElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_status_attribute(&attr)? {
                    builder.status_attribute(value);
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
                    b"p" => {
                        let recognized = read_p_element(reader, start)?;
                        builder.p_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"availability" => {
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
    #[builder(setter(custom), default)]
    pub ref_elements: Vec<RefElement>,
    #[builder(setter(custom), default)]
    pub ptr_element: Option<PtrElement>,
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

impl PElementBuilder {
    pub fn ref_element(&mut self, value: RefElement){
        let targ = self.ref_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn ptr_element(&mut self, value: PtrElement){
        assert!(self.ptr_element.is_none(), "ptr_element in PElement should be unset!");
        self.ptr_element = Some(Some(value));
    }
}

pub struct PElementIterFunction;

impl iter::IterHelper<PElement, TeiReaderError> for PElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PElement>, TeiReaderError> {
        read_as_root_p_element(reader)
    }
}

pub fn iter_for_p_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_p_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PElement>, TeiReaderError>{
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

pub fn read_p_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"ref" => {
                        let recognized = read_ref_element(reader, start)?;
                        builder.ref_element(recognized);
                    }
                    b"ptr" => {
                        let recognized = read_ptr_element(reader, start)?;
                        builder.ptr_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"p" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"ref" => {
                        let recognized = read_ref_element(reader, value)?;
                        builder.ref_element(recognized);
                    }
                    b"ptr" => {
                        let recognized = read_ptr_element(reader, value)?;
                        builder.ptr_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
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


// Element - ref - ref
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct RefElement {
    #[builder(setter(strip_option), default)]
    pub target_attribute: Option<String>,
    pub content: String,
}

pub struct RefElementIterFunction;

impl iter::IterHelper<RefElement, TeiReaderError> for RefElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RefElement>, TeiReaderError> {
        read_as_root_ref_element(reader)
    }
}

pub fn iter_for_ref_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::RefElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_ref_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RefElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"ref" => {
                        break Ok(Some(read_ref_element(reader, start)?))
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

pub fn read_ref_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<RefElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = RefElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_target_attribute(&attr)? {
                    builder.target_attribute(value);
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
                    b"ref" => {
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


// Element - date - date
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct DateElement {
    pub when_attribute: String,
    pub content: String,
}

pub struct DateElementIterFunction;

impl iter::IterHelper<DateElement, TeiReaderError> for DateElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DateElement>, TeiReaderError> {
        read_as_root_date_element(reader)
    }
}

pub fn iter_for_date_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::DateElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_date_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<DateElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"date" => {
                        break Ok(Some(read_date_element(reader, start)?))
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

pub fn read_date_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<DateElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = DateElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_when_attribute(&attr)? {
                    builder.when_attribute(value);
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
                    b"date" => {
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


// Element - pubPlace - pubPlace
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PubPlaceElement {
    #[builder(setter(custom), default)]
    pub ref_element: Option<RefElement>,
}

impl PubPlaceElementBuilder {
    pub fn ref_element(&mut self, value: RefElement){
        assert!(self.ref_element.is_none(), "ref_element in PubPlaceElement should be unset!");
        self.ref_element = Some(Some(value));
    }
}

pub struct PubPlaceElementIterFunction;

impl iter::IterHelper<PubPlaceElement, TeiReaderError> for PubPlaceElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PubPlaceElement>, TeiReaderError> {
        read_as_root_pub_place_element(reader)
    }
}

pub fn iter_for_pub_place_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PubPlaceElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_pub_place_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PubPlaceElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"pubPlace" => {
                        break Ok(Some(read_pub_place_element(reader, start)?))
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

pub fn read_pub_place_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PubPlaceElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PubPlaceElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"ref" => {
                        let recognized = read_ref_element(reader, start)?;
                        builder.ref_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"pubPlace" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"ref" => {
                        let recognized = read_ref_element(reader, value)?;
                        builder.ref_element(recognized);
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


// Element - notesStmt - notesStmt
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct NotesStmtElement {
    #[builder(setter(custom), default)]
    pub note_element: Option<NoteElement>,
}

impl NotesStmtElementBuilder {
    pub fn note_element(&mut self, value: NoteElement){
        assert!(self.note_element.is_none(), "note_element in NotesStmtElement should be unset!");
        self.note_element = Some(Some(value));
    }
}

pub struct NotesStmtElementIterFunction;

impl iter::IterHelper<NotesStmtElement, TeiReaderError> for NotesStmtElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NotesStmtElement>, TeiReaderError> {
        read_as_root_notes_stmt_element(reader)
    }
}

pub fn iter_for_notes_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::NotesStmtElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_notes_stmt_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NotesStmtElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"notesStmt" => {
                        break Ok(Some(read_notes_stmt_element(reader, start)?))
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

pub fn read_notes_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<NotesStmtElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = NotesStmtElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"note" => {
                        let recognized = read_note_element(reader, start)?;
                        builder.note_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"notesStmt" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"note" => {
                        let recognized = read_note_element(reader, value)?;
                        builder.note_element(recognized);
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


// Element - note - note
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct NoteElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub struct NoteElementIterFunction;

impl iter::IterHelper<NoteElement, TeiReaderError> for NoteElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NoteElement>, TeiReaderError> {
        read_as_root_note_element(reader)
    }
}

pub fn iter_for_note_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::NoteElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_note_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NoteElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"note" => {
                        break Ok(Some(read_note_element(reader, start)?))
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

pub fn read_note_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<NoteElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = NoteElementBuilder::create_empty();
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
                    b"note" => {
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
    #[builder(setter(custom), default)]
    pub p_elements: Vec<PElement>,
}

impl SourceDescElementBuilder {
    pub fn p_element(&mut self, value: PElement){
        let targ = self.p_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct SourceDescElementIterFunction;

impl iter::IterHelper<SourceDescElement, TeiReaderError> for SourceDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SourceDescElement>, TeiReaderError> {
        read_as_root_source_desc_element(reader)
    }
}

pub fn iter_for_source_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::SourceDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_source_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SourceDescElement>, TeiReaderError>{
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

pub fn read_source_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<SourceDescElement, TeiReaderError>{
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


// Element - ptr - ptr
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PtrElement {
    #[builder(setter(strip_option), default)]
    pub target_attribute: Option<String>,
}

pub struct PtrElementIterFunction;

impl iter::IterHelper<PtrElement, TeiReaderError> for PtrElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PtrElement>, TeiReaderError> {
        read_as_root_ptr_element(reader)
    }
}

pub fn iter_for_ptr_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PtrElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_ptr_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PtrElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"ptr" => {
                        break Ok(Some(read_ptr_element(reader, start)?))
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

pub fn read_ptr_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<PtrElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PtrElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_target_attribute(&attr)? {
                    builder.target_attribute(value);
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
                    b"ptr" => {
                        break;
                    }
                    _ => {}                }
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


// Element - encodingDesc - encodingDesc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct EncodingDescElement {
    #[builder(setter(custom), default)]
    pub project_desc_element: Option<ProjectDescElement>,
}

impl EncodingDescElementBuilder {
    pub fn project_desc_element(&mut self, value: ProjectDescElement){
        assert!(self.project_desc_element.is_none(), "project_desc_element in EncodingDescElement should be unset!");
        self.project_desc_element = Some(Some(value));
    }
}

pub struct EncodingDescElementIterFunction;

impl iter::IterHelper<EncodingDescElement, TeiReaderError> for EncodingDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EncodingDescElement>, TeiReaderError> {
        read_as_root_encoding_desc_element(reader)
    }
}

pub fn iter_for_encoding_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::EncodingDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_encoding_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EncodingDescElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"encodingDesc" => {
                        break Ok(Some(read_encoding_desc_element(reader, start)?))
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

pub fn read_encoding_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<EncodingDescElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = EncodingDescElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"projectDesc" => {
                        let recognized = read_project_desc_element(reader, start)?;
                        builder.project_desc_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"encodingDesc" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"projectDesc" => {
                        let recognized = read_project_desc_element(reader, value)?;
                        builder.project_desc_element(recognized);
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


// Element - projectDesc - projectDesc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct ProjectDescElement {
    #[builder(setter(custom), default)]
    pub p_element: Option<PElement>,
}

impl ProjectDescElementBuilder {
    pub fn p_element(&mut self, value: PElement){
        assert!(self.p_element.is_none(), "p_element in ProjectDescElement should be unset!");
        self.p_element = Some(Some(value));
    }
}

pub struct ProjectDescElementIterFunction;

impl iter::IterHelper<ProjectDescElement, TeiReaderError> for ProjectDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ProjectDescElement>, TeiReaderError> {
        read_as_root_project_desc_element(reader)
    }
}

pub fn iter_for_project_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::ProjectDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_project_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ProjectDescElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"projectDesc" => {
                        break Ok(Some(read_project_desc_element(reader, start)?))
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

pub fn read_project_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<ProjectDescElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = ProjectDescElementBuilder::create_empty();
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
                    b"projectDesc" => {
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


// Element - revisionDesc - revisionDesc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct RevisionDescElement {
    #[builder(setter(custom), default)]
    pub change_elements: Vec<ChangeElement>,
}

impl RevisionDescElementBuilder {
    pub fn change_element(&mut self, value: ChangeElement){
        let targ = self.change_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct RevisionDescElementIterFunction;

impl iter::IterHelper<RevisionDescElement, TeiReaderError> for RevisionDescElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RevisionDescElement>, TeiReaderError> {
        read_as_root_revision_desc_element(reader)
    }
}

pub fn iter_for_revision_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::RevisionDescElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_revision_desc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<RevisionDescElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"revisionDesc" => {
                        break Ok(Some(read_revision_desc_element(reader, start)?))
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

pub fn read_revision_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<RevisionDescElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = RevisionDescElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"change" => {
                        let recognized = read_change_element(reader, start)?;
                        builder.change_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"revisionDesc" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"change" => {
                        let recognized = read_change_element(reader, value)?;
                        builder.change_element(recognized);
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


// Element - change - change
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct ChangeElement {
    #[builder(setter(strip_option), default)]
    pub when_attribute: Option<String>,
    pub who_attribute: String,
    pub n_attribute: String,
    #[builder(setter(custom), default)]
    pub list_element: Option<ListElement>,
}

impl ChangeElementBuilder {
    pub fn list_element(&mut self, value: ListElement){
        assert!(self.list_element.is_none(), "list_element in ChangeElement should be unset!");
        self.list_element = Some(Some(value));
    }
}

pub struct ChangeElementIterFunction;

impl iter::IterHelper<ChangeElement, TeiReaderError> for ChangeElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ChangeElement>, TeiReaderError> {
        read_as_root_change_element(reader)
    }
}

pub fn iter_for_change_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::ChangeElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_change_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ChangeElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"change" => {
                        break Ok(Some(read_change_element(reader, start)?))
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

pub fn read_change_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<ChangeElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = ChangeElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_when_attribute(&attr)? {
                    builder.when_attribute(value);
                    continue;
                }
                if let Some(value) = read_who_attribute(&attr)? {
                    builder.who_attribute(value);
                    continue;
                }
                if let Some(value) = read_n_attribute(&attr)? {
                    builder.n_attribute(value);
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
                    b"list" => {
                        let recognized = read_list_element(reader, start)?;
                        builder.list_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"change" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"list" => {
                        let recognized = read_list_element(reader, value)?;
                        builder.list_element(recognized);
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


// Element - list - list
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct ListElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(custom), default)]
    pub item_elements: Vec<ItemElement>,
    #[builder(setter(custom), default)]
    pub head_element: Option<HeadElement>,
}

impl ListElementBuilder {
    pub fn item_element(&mut self, value: ItemElement){
        let targ = self.item_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn head_element(&mut self, value: HeadElement){
        assert!(self.head_element.is_none(), "head_element in ListElement should be unset!");
        self.head_element = Some(Some(value));
    }
}

pub struct ListElementIterFunction;

impl iter::IterHelper<ListElement, TeiReaderError> for ListElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ListElement>, TeiReaderError> {
        read_as_root_list_element(reader)
    }
}

pub fn iter_for_list_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::ListElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_list_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ListElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"list" => {
                        break Ok(Some(read_list_element(reader, start)?))
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

pub fn read_list_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<ListElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = ListElementBuilder::create_empty();
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
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"item" => {
                        let recognized = read_item_element(reader, start)?;
                        builder.item_element(recognized);
                    }
                    b"head" => {
                        let recognized = read_head_element(reader, start)?;
                        builder.head_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"list" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"item" => {
                        let recognized = read_item_element(reader, value)?;
                        builder.item_element(recognized);
                    }
                    b"head" => {
                        let recognized = read_head_element(reader, value)?;
                        builder.head_element(recognized);
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


// Element - head - head
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct HeadElement {
    #[builder(setter(custom), default)]
    pub name_element: Option<NameElement>,
    #[builder(setter(custom), default)]
    pub date_element: Option<DateElement>,
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

impl HeadElementBuilder {
    pub fn name_element(&mut self, value: NameElement){
        assert!(self.name_element.is_none(), "name_element in HeadElement should be unset!");
        self.name_element = Some(Some(value));
    }
    pub fn date_element(&mut self, value: DateElement){
        assert!(self.date_element.is_none(), "date_element in HeadElement should be unset!");
        self.date_element = Some(Some(value));
    }
}

pub struct HeadElementIterFunction;

impl iter::IterHelper<HeadElement, TeiReaderError> for HeadElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<HeadElement>, TeiReaderError> {
        read_as_root_head_element(reader)
    }
}

pub fn iter_for_head_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::HeadElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_head_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<HeadElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"head" => {
                        break Ok(Some(read_head_element(reader, start)?))
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

pub fn read_head_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<HeadElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = HeadElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"name" => {
                        let recognized = read_name_element(reader, start)?;
                        builder.name_element(recognized);
                    }
                    b"date" => {
                        let recognized = read_date_element(reader, start)?;
                        builder.date_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"head" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"name" => {
                        let recognized = read_name_element(reader, value)?;
                        builder.name_element(recognized);
                    }
                    b"date" => {
                        let recognized = read_date_element(reader, value)?;
                        builder.date_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
                break;
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


// Element - item - item
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct ItemElement {
    pub content: String,
}

pub struct ItemElementIterFunction;

impl iter::IterHelper<ItemElement, TeiReaderError> for ItemElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ItemElement>, TeiReaderError> {
        read_as_root_item_element(reader)
    }
}

pub fn iter_for_item_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::ItemElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_item_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<ItemElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"item" => {
                        break Ok(Some(read_item_element(reader, start)?))
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

pub fn read_item_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<ItemElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = ItemElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"item" => {
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
    pub lang_attribute: Option<LangAttribute>,
    #[builder(setter(custom), default)]
    pub body_element: Option<BodyElement>,
}

impl TextElementBuilder {
    pub fn body_element(&mut self, value: BodyElement){
        assert!(self.body_element.is_none(), "body_element in TextElement should be unset!");
        self.body_element = Some(Some(value));
    }
}

pub struct TextElementIterFunction;

impl iter::IterHelper<TextElement, TeiReaderError> for TextElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TextElement>, TeiReaderError> {
        read_as_root_text_element(reader)
    }
}

pub fn iter_for_text_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TextElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_text_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TextElement>, TeiReaderError>{
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

pub fn read_text_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<TextElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TextElementBuilder::create_empty();
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
    pub entry_elements: Vec<EntryElement>,
}

impl BodyElementBuilder {
    pub fn entry_element(&mut self, value: EntryElement){
        let targ = self.entry_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct BodyElementIterFunction;

impl iter::IterHelper<BodyElement, TeiReaderError> for BodyElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<BodyElement>, TeiReaderError> {
        read_as_root_body_element(reader)
    }
}

pub fn iter_for_body_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::BodyElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_body_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<BodyElement>, TeiReaderError>{
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

pub fn read_body_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<BodyElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = BodyElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"entry" => {
                        let recognized = read_entry_element(reader, start)?;
                        builder.entry_element(recognized);
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
                    b"entry" => {
                        let recognized = read_entry_element(reader, value)?;
                        builder.entry_element(recognized);
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


// Element - entry - entry
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct EntryElement {
    pub id_attribute: String,
    #[builder(setter(custom), default)]
    pub sense_element: Option<SenseElement>,
    #[builder(setter(custom), default)]
    pub gram_grp_element: Option<GramGrpElement>,
    #[builder(setter(custom), default)]
    pub form_element: Option<FormElement>,
}

impl EntryElementBuilder {
    pub fn sense_element(&mut self, value: SenseElement){
        assert!(self.sense_element.is_none(), "sense_element in EntryElement should be unset!");
        self.sense_element = Some(Some(value));
    }
    pub fn gram_grp_element(&mut self, value: GramGrpElement){
        assert!(self.gram_grp_element.is_none(), "gram_grp_element in EntryElement should be unset!");
        self.gram_grp_element = Some(Some(value));
    }
    pub fn form_element(&mut self, value: FormElement){
        assert!(self.form_element.is_none(), "form_element in EntryElement should be unset!");
        self.form_element = Some(Some(value));
    }
}

pub struct EntryElementIterFunction;

impl iter::IterHelper<EntryElement, TeiReaderError> for EntryElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EntryElement>, TeiReaderError> {
        read_as_root_entry_element(reader)
    }
}

pub fn iter_for_entry_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::EntryElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_entry_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<EntryElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"entry" => {
                        break Ok(Some(read_entry_element(reader, start)?))
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

pub fn read_entry_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<EntryElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = EntryElementBuilder::create_empty();
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
                    b"sense" => {
                        let recognized = read_sense_element(reader, start)?;
                        builder.sense_element(recognized);
                    }
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, start)?;
                        builder.gram_grp_element(recognized);
                    }
                    b"form" => {
                        let recognized = read_form_element(reader, start)?;
                        builder.form_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"entry" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"sense" => {
                        let recognized = read_sense_element(reader, value)?;
                        builder.sense_element(recognized);
                    }
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, value)?;
                        builder.gram_grp_element(recognized);
                    }
                    b"form" => {
                        let recognized = read_form_element(reader, value)?;
                        builder.form_element(recognized);
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


// Element - form - form
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct FormElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(custom), default)]
    pub gram_grp_element: Option<GramGrpElement>,
    #[builder(setter(custom), default)]
    pub orth_element: Option<OrthElement>,
    #[builder(setter(custom), default)]
    pub form_elements: Vec<FormElement>,
    #[builder(setter(custom), default)]
    pub usg_element: Option<UsgElement>,
}

impl FormElementBuilder {
    pub fn gram_grp_element(&mut self, value: GramGrpElement){
        assert!(self.gram_grp_element.is_none(), "gram_grp_element in FormElement should be unset!");
        self.gram_grp_element = Some(Some(value));
    }
    pub fn orth_element(&mut self, value: OrthElement){
        assert!(self.orth_element.is_none(), "orth_element in FormElement should be unset!");
        self.orth_element = Some(Some(value));
    }
    pub fn form_element(&mut self, value: FormElement){
        let targ = self.form_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn usg_element(&mut self, value: UsgElement){
        assert!(self.usg_element.is_none(), "usg_element in FormElement should be unset!");
        self.usg_element = Some(Some(value));
    }
}

pub struct FormElementIterFunction;

impl iter::IterHelper<FormElement, TeiReaderError> for FormElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FormElement>, TeiReaderError> {
        read_as_root_form_element(reader)
    }
}

pub fn iter_for_form_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::FormElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_form_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<FormElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"form" => {
                        break Ok(Some(read_form_element(reader, start)?))
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

pub fn read_form_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<FormElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = FormElementBuilder::create_empty();
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
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, start)?;
                        builder.gram_grp_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, start)?;
                        builder.orth_element(recognized);
                    }
                    b"form" => {
                        let recognized = read_form_element(reader, start)?;
                        builder.form_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, start)?;
                        builder.usg_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"form" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, value)?;
                        builder.gram_grp_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, value)?;
                        builder.orth_element(recognized);
                    }
                    b"form" => {
                        let recognized = read_form_element(reader, value)?;
                        builder.form_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, value)?;
                        builder.usg_element(recognized);
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


// Element - orth - orth
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct OrthElement {
    pub content: String,
}

pub struct OrthElementIterFunction;

impl iter::IterHelper<OrthElement, TeiReaderError> for OrthElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<OrthElement>, TeiReaderError> {
        read_as_root_orth_element(reader)
    }
}

pub fn iter_for_orth_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::OrthElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_orth_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<OrthElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"orth" => {
                        break Ok(Some(read_orth_element(reader, start)?))
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

pub fn read_orth_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<OrthElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = OrthElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"orth" => {
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


// Element - gramGrp - gramGrp
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct GramGrpElement {
    #[builder(setter(custom), default)]
    pub gen_elements: Vec<GenElement>,
    #[builder(setter(custom), default)]
    pub pos_elements: Vec<PosElement>,
    #[builder(setter(custom), default)]
    pub tns_element: Option<TnsElement>,
    #[builder(setter(custom), default)]
    pub colloc_elements: Vec<CollocElement>,
    #[builder(setter(custom), default)]
    pub subc_elements: Vec<SubcElement>,
    #[builder(setter(custom), default)]
    pub mood_element: Option<MoodElement>,
    #[builder(setter(custom), default)]
    pub number_elements: Vec<NumberElement>,
}

impl GramGrpElementBuilder {
    pub fn gen_element(&mut self, value: GenElement){
        let targ = self.gen_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn pos_element(&mut self, value: PosElement){
        let targ = self.pos_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn tns_element(&mut self, value: TnsElement){
        assert!(self.tns_element.is_none(), "tns_element in GramGrpElement should be unset!");
        self.tns_element = Some(Some(value));
    }
    pub fn colloc_element(&mut self, value: CollocElement){
        let targ = self.colloc_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn subc_element(&mut self, value: SubcElement){
        let targ = self.subc_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn mood_element(&mut self, value: MoodElement){
        assert!(self.mood_element.is_none(), "mood_element in GramGrpElement should be unset!");
        self.mood_element = Some(Some(value));
    }
    pub fn number_element(&mut self, value: NumberElement){
        let targ = self.number_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct GramGrpElementIterFunction;

impl iter::IterHelper<GramGrpElement, TeiReaderError> for GramGrpElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<GramGrpElement>, TeiReaderError> {
        read_as_root_gram_grp_element(reader)
    }
}

pub fn iter_for_gram_grp_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::GramGrpElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_gram_grp_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<GramGrpElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"gramGrp" => {
                        break Ok(Some(read_gram_grp_element(reader, start)?))
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

pub fn read_gram_grp_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<GramGrpElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = GramGrpElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"gen" => {
                        let recognized = read_gen_element(reader, start)?;
                        builder.gen_element(recognized);
                    }
                    b"pos" => {
                        let recognized = read_pos_element(reader, start)?;
                        builder.pos_element(recognized);
                    }
                    b"tns" => {
                        let recognized = read_tns_element(reader, start)?;
                        builder.tns_element(recognized);
                    }
                    b"colloc" => {
                        let recognized = read_colloc_element(reader, start)?;
                        builder.colloc_element(recognized);
                    }
                    b"subc" => {
                        let recognized = read_subc_element(reader, start)?;
                        builder.subc_element(recognized);
                    }
                    b"mood" => {
                        let recognized = read_mood_element(reader, start)?;
                        builder.mood_element(recognized);
                    }
                    b"number" => {
                        let recognized = read_number_element(reader, start)?;
                        builder.number_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"gramGrp" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"gen" => {
                        let recognized = read_gen_element(reader, value)?;
                        builder.gen_element(recognized);
                    }
                    b"pos" => {
                        let recognized = read_pos_element(reader, value)?;
                        builder.pos_element(recognized);
                    }
                    b"tns" => {
                        let recognized = read_tns_element(reader, value)?;
                        builder.tns_element(recognized);
                    }
                    b"colloc" => {
                        let recognized = read_colloc_element(reader, value)?;
                        builder.colloc_element(recognized);
                    }
                    b"subc" => {
                        let recognized = read_subc_element(reader, value)?;
                        builder.subc_element(recognized);
                    }
                    b"mood" => {
                        let recognized = read_mood_element(reader, value)?;
                        builder.mood_element(recognized);
                    }
                    b"number" => {
                        let recognized = read_number_element(reader, value)?;
                        builder.number_element(recognized);
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum EGenElement {
    #[strum(serialize="neut")]
    Neut,
    #[strum(serialize="masc")]
    Masc,
    #[strum(serialize="fem")]
    Fem,
}

// Element - gen - gen
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct GenElement {
    pub content: EGenElement,
}

pub struct GenElementIterFunction;

impl iter::IterHelper<GenElement, TeiReaderError> for GenElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<GenElement>, TeiReaderError> {
        read_as_root_gen_element(reader)
    }
}

pub fn iter_for_gen_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::GenElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_gen_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<GenElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"gen" => {
                        break Ok(Some(read_gen_element(reader, start)?))
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

pub fn read_gen_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<GenElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = GenElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"gen" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(TeiReaderError::ElementStrumParserError("gen", error, s.to_string()));
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum EPosElement {
    #[strum(serialize="n")]
    N,
    #[strum(serialize="adj")]
    Adj,
    #[strum(serialize="v")]
    V,
    #[strum(serialize="adv")]
    Adv,
    #[strum(serialize="int")]
    Int,
    #[strum(serialize="prep")]
    Prep,
    #[strum(serialize="num")]
    Num,
    #[strum(serialize="pron")]
    Pron,
    #[strum(serialize="conj")]
    Conj,
    #[strum(serialize="art")]
    Art,
    #[strum(serialize="ptcl")]
    Ptcl,
}

// Element - pos - pos
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PosElement {
    pub content: EPosElement,
}

pub struct PosElementIterFunction;

impl iter::IterHelper<PosElement, TeiReaderError> for PosElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PosElement>, TeiReaderError> {
        read_as_root_pos_element(reader)
    }
}

pub fn iter_for_pos_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::PosElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_pos_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<PosElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"pos" => {
                        break Ok(Some(read_pos_element(reader, start)?))
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

pub fn read_pos_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PosElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PosElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"pos" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(TeiReaderError::ElementStrumParserError("pos", error, s.to_string()));
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum ENumberElement {
    #[strum(serialize="sg")]
    Sg,
    #[strum(serialize="pl")]
    Pl,
}

// Element - number - number
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct NumberElement {
    pub content: ENumberElement,
}

pub struct NumberElementIterFunction;

impl iter::IterHelper<NumberElement, TeiReaderError> for NumberElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NumberElement>, TeiReaderError> {
        read_as_root_number_element(reader)
    }
}

pub fn iter_for_number_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::NumberElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_number_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<NumberElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"number" => {
                        break Ok(Some(read_number_element(reader, start)?))
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

pub fn read_number_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<NumberElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = NumberElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"number" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(TeiReaderError::ElementStrumParserError("number", error, s.to_string()));
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


// Element - sense - sense
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct SenseElement {
    #[builder(setter(custom), default)]
    pub xr_elements: Vec<XrElement>,
    #[builder(setter(custom), default)]
    pub usg_elements: Vec<UsgElement>,
    #[builder(setter(custom), default)]
    pub note_elements: Vec<NoteElement>,
    #[builder(setter(custom), default)]
    pub cit_elements: Vec<CitElement>,
}

impl SenseElementBuilder {
    pub fn xr_element(&mut self, value: XrElement){
        let targ = self.xr_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn usg_element(&mut self, value: UsgElement){
        let targ = self.usg_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn note_element(&mut self, value: NoteElement){
        let targ = self.note_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn cit_element(&mut self, value: CitElement){
        let targ = self.cit_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct SenseElementIterFunction;

impl iter::IterHelper<SenseElement, TeiReaderError> for SenseElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SenseElement>, TeiReaderError> {
        read_as_root_sense_element(reader)
    }
}

pub fn iter_for_sense_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::SenseElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_sense_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SenseElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"sense" => {
                        break Ok(Some(read_sense_element(reader, start)?))
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

pub fn read_sense_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<SenseElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = SenseElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"xr" => {
                        let recognized = read_xr_element(reader, start)?;
                        builder.xr_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, start)?;
                        builder.usg_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, start)?;
                        builder.note_element(recognized);
                    }
                    b"cit" => {
                        let recognized = read_cit_element(reader, start)?;
                        builder.cit_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"sense" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"xr" => {
                        let recognized = read_xr_element(reader, value)?;
                        builder.xr_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, value)?;
                        builder.usg_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, value)?;
                        builder.note_element(recognized);
                    }
                    b"cit" => {
                        let recognized = read_cit_element(reader, value)?;
                        builder.cit_element(recognized);
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


// Element - cit - cit
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct CitElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(custom), default)]
    pub cit_elements: Vec<CitElement>,
    #[builder(setter(custom), default)]
    pub usg_elements: Vec<UsgElement>,
    #[builder(setter(custom), default)]
    pub note_elements: Vec<NoteElement>,
    #[builder(setter(custom), default)]
    pub quote_element: Option<QuoteElement>,
    #[builder(setter(custom), default)]
    pub orth_element: Option<OrthElement>,
    #[builder(setter(custom), default)]
    pub gram_grp_element: Option<GramGrpElement>,
}

impl CitElementBuilder {
    pub fn cit_element(&mut self, value: CitElement){
        let targ = self.cit_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn usg_element(&mut self, value: UsgElement){
        let targ = self.usg_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn note_element(&mut self, value: NoteElement){
        let targ = self.note_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn quote_element(&mut self, value: QuoteElement){
        assert!(self.quote_element.is_none(), "quote_element in CitElement should be unset!");
        self.quote_element = Some(Some(value));
    }
    pub fn orth_element(&mut self, value: OrthElement){
        assert!(self.orth_element.is_none(), "orth_element in CitElement should be unset!");
        self.orth_element = Some(Some(value));
    }
    pub fn gram_grp_element(&mut self, value: GramGrpElement){
        assert!(self.gram_grp_element.is_none(), "gram_grp_element in CitElement should be unset!");
        self.gram_grp_element = Some(Some(value));
    }
}

pub struct CitElementIterFunction;

impl iter::IterHelper<CitElement, TeiReaderError> for CitElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<CitElement>, TeiReaderError> {
        read_as_root_cit_element(reader)
    }
}

pub fn iter_for_cit_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::CitElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_cit_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<CitElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"cit" => {
                        break Ok(Some(read_cit_element(reader, start)?))
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

pub fn read_cit_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<CitElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = CitElementBuilder::create_empty();
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
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"cit" => {
                        let recognized = read_cit_element(reader, start)?;
                        builder.cit_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, start)?;
                        builder.usg_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, start)?;
                        builder.note_element(recognized);
                    }
                    b"quote" => {
                        let recognized = read_quote_element(reader, start)?;
                        builder.quote_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, start)?;
                        builder.orth_element(recognized);
                    }
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, start)?;
                        builder.gram_grp_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"cit" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"cit" => {
                        let recognized = read_cit_element(reader, value)?;
                        builder.cit_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, value)?;
                        builder.usg_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, value)?;
                        builder.note_element(recognized);
                    }
                    b"quote" => {
                        let recognized = read_quote_element(reader, value)?;
                        builder.quote_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, value)?;
                        builder.orth_element(recognized);
                    }
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, value)?;
                        builder.gram_grp_element(recognized);
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


// Element - quote - quote
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct QuoteElement {
    #[builder(setter(strip_option), default)]
    pub lang_attribute: Option<LangAttribute>,
    pub content: String,
}

pub struct QuoteElementIterFunction;

impl iter::IterHelper<QuoteElement, TeiReaderError> for QuoteElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<QuoteElement>, TeiReaderError> {
        read_as_root_quote_element(reader)
    }
}

pub fn iter_for_quote_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::QuoteElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_quote_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<QuoteElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"quote" => {
                        break Ok(Some(read_quote_element(reader, start)?))
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

pub fn read_quote_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<QuoteElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = QuoteElementBuilder::create_empty();
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
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"quote" => {
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


// Element - xr - xr
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct XrElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(custom), default)]
    pub ref_elements: Vec<RefElement>,
}

impl XrElementBuilder {
    pub fn ref_element(&mut self, value: RefElement){
        let targ = self.ref_elements.get_or_insert_with(Default::default);
        targ.push(value);
    }
}

pub struct XrElementIterFunction;

impl iter::IterHelper<XrElement, TeiReaderError> for XrElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<XrElement>, TeiReaderError> {
        read_as_root_xr_element(reader)
    }
}

pub fn iter_for_xr_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::XrElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_xr_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<XrElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"xr" => {
                        break Ok(Some(read_xr_element(reader, start)?))
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

pub fn read_xr_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<XrElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = XrElementBuilder::create_empty();
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
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"ref" => {
                        let recognized = read_ref_element(reader, start)?;
                        builder.ref_element(recognized);
                    }
                    unknown => { log::warn!("Unknown Tag: '{}'", String::from_utf8_lossy(unknown)); }
                }
            }
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"xr" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Empty(value) => {
                
                match value.local_name().as_ref(){
                    b"ref" => {
                        let recognized = read_ref_element(reader, value)?;
                        builder.ref_element(recognized);
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


// Element - usg - usg
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct UsgElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    pub content: String,
}

pub struct UsgElementIterFunction;

impl iter::IterHelper<UsgElement, TeiReaderError> for UsgElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<UsgElement>, TeiReaderError> {
        read_as_root_usg_element(reader)
    }
}

pub fn iter_for_usg_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::UsgElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_usg_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<UsgElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"usg" => {
                        break Ok(Some(read_usg_element(reader, start)?))
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

pub fn read_usg_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<UsgElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = UsgElementBuilder::create_empty();
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
                    b"usg" => {
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


// Element - subc - subc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct SubcElement {
    pub content: String,
}

pub struct SubcElementIterFunction;

impl iter::IterHelper<SubcElement, TeiReaderError> for SubcElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SubcElement>, TeiReaderError> {
        read_as_root_subc_element(reader)
    }
}

pub fn iter_for_subc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::SubcElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_subc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<SubcElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"subc" => {
                        break Ok(Some(read_subc_element(reader, start)?))
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

pub fn read_subc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<SubcElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = SubcElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"subc" => {
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


// Element - colloc - colloc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct CollocElement {
    pub content: String,
}

pub struct CollocElementIterFunction;

impl iter::IterHelper<CollocElement, TeiReaderError> for CollocElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<CollocElement>, TeiReaderError> {
        read_as_root_colloc_element(reader)
    }
}

pub fn iter_for_colloc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::CollocElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_colloc_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<CollocElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"colloc" => {
                        break Ok(Some(read_colloc_element(reader, start)?))
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

pub fn read_colloc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<CollocElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = CollocElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"colloc" => {
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum ETnsElement {
    #[strum(serialize="past")]
    Past,
    #[strum(serialize="pstp")]
    Pstp,
}

// Element - tns - tns
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TnsElement {
    pub content: ETnsElement,
}

pub struct TnsElementIterFunction;

impl iter::IterHelper<TnsElement, TeiReaderError> for TnsElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TnsElement>, TeiReaderError> {
        read_as_root_tns_element(reader)
    }
}

pub fn iter_for_tns_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::TnsElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_tns_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<TnsElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"tns" => {
                        break Ok(Some(read_tns_element(reader, start)?))
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

pub fn read_tns_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TnsElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = TnsElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"tns" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(TeiReaderError::ElementStrumParserError("tns", error, s.to_string()));
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum EMoodElement {
    #[strum(serialize="indicative")]
    Indicative,
}

// Element - mood - mood
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct MoodElement {
    pub content: EMoodElement,
}

pub struct MoodElementIterFunction;

impl iter::IterHelper<MoodElement, TeiReaderError> for MoodElementIterFunction {
    #[inline(always)]
    fn goto_next<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<MoodElement>, TeiReaderError> {
        read_as_root_mood_element(reader)
    }
}

pub fn iter_for_mood_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::MoodElementIter<R>{
    iter::Iter::new(reader)
}

pub fn read_as_root_mood_element<R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<MoodElement>, TeiReaderError>{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"mood" => {
                        break Ok(Some(read_mood_element(reader, start)?))
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

pub fn read_mood_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<MoodElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = MoodElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::End(value) => {
                match value.name().local_name().as_ref() {
                    b"mood" => {
                        break;
                    }
                    _ => {}                }
            }
            quick_xml::events::Event::Text(value) => {
                let s_value = std::str::from_utf8(value.as_ref())?;
                let s = s_value.trim();
                match s.parse(){
                    Ok(value) => {
                        builder.content(value);
                    }
                    Err(error) => {
                        return Err(TeiReaderError::ElementStrumParserError("mood", error, s.to_string()));
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


// Attribute - xmlns - XmlnsAttribute
pub fn read_xmlns_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"xmlns" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
}

// Attribute - version - VersionAttribute
pub fn read_version_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<f64>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"version" {
        let value = attr.unescape_value()?;
        Ok(Some(value.trim().to_lowercase().as_str().parse()?))
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
pub fn read_lang_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<LangAttribute>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"lang" {
        let value = attr.unescape_value()?;
        let s = value.trim().to_lowercase();
        match s.parse(){
            Ok(value) => Ok(Some(value)),
            Err(error) => Err(TeiReaderError::AttributeStrumParserError("lang", error, s)),
        }
    } else { Ok(None) }
}

// Attribute - status - StatusAttribute
#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum StatusAttribute {
    #[strum(serialize="free")]
    Free,
}

// Attribute - status - StatusAttribute
pub fn read_status_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<StatusAttribute>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"status" {
        let value = attr.unescape_value()?;
        let s = value.trim().to_lowercase();
        match s.parse(){
            Ok(value) => Ok(Some(value)),
            Err(error) => Err(TeiReaderError::AttributeStrumParserError("status", error, s)),
        }
    } else { Ok(None) }
}

// Attribute - target - TargetAttribute
pub fn read_target_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"target" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
}

// Attribute - when - WhenAttribute
pub fn read_when_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"when" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
}

// Attribute - type - TypeAttribute
#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum TypeAttribute {
    #[strum(serialize="colloc")]
    Colloc,
    #[strum(serialize="example")]
    Example,
    #[strum(serialize="abbrev")]
    Abbrev,
    #[strum(serialize="syn")]
    Syn,
    #[strum(serialize="status")]
    Status,
    #[strum(serialize="time")]
    Time,
    #[strum(serialize="lang")]
    Lang,
    #[strum(serialize="bulleted")]
    Bulleted,
    #[strum(serialize="hint")]
    Hint,
    #[strum(serialize="style")]
    Style,
    #[strum(serialize="see")]
    See,
    #[strum(serialize="trans")]
    Trans,
    #[strum(serialize="dom")]
    Dom,
    #[strum(serialize="geo")]
    Geo,
    #[strum(serialize="infl")]
    Infl,
    #[strum(serialize="reg")]
    Reg,
}

// Attribute - type - TypeAttribute
pub fn read_type_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<TypeAttribute>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"type" {
        let value = attr.unescape_value()?;
        let s = value.trim().to_lowercase();
        match s.parse(){
            Ok(value) => Ok(Some(value)),
            Err(error) => Err(TeiReaderError::AttributeStrumParserError("type", error, s)),
        }
    } else { Ok(None) }
}

// Attribute - n - NAttribute
pub fn read_n_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"n" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
}

// Attribute - who - WhoAttribute
pub fn read_who_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"who" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
}

// Attribute - id - IdAttribute
pub fn read_id_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"id" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))    } else { Ok(None) }
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

    use super::TeiReaderError;

    use super::TeiElement;
    use super::TeiElementIterFunction;
    /// Iterator for TeiElement
    pub type TeiElementIter<R> = Iter<R, TeiElement, TeiReaderError, TeiElementIterFunction>;

    use super::TeiHeaderElement;
    use super::TeiHeaderElementIterFunction;
    /// Iterator for TeiHeaderElement
    pub type TeiHeaderElementIter<R> = Iter<R, TeiHeaderElement, TeiReaderError, TeiHeaderElementIterFunction>;

    use super::FileDescElement;
    use super::FileDescElementIterFunction;
    /// Iterator for FileDescElement
    pub type FileDescElementIter<R> = Iter<R, FileDescElement, TeiReaderError, FileDescElementIterFunction>;

    use super::TitleStmtElement;
    use super::TitleStmtElementIterFunction;
    /// Iterator for TitleStmtElement
    pub type TitleStmtElementIter<R> = Iter<R, TitleStmtElement, TeiReaderError, TitleStmtElementIterFunction>;

    use super::TitleElement;
    use super::TitleElementIterFunction;
    /// Iterator for TitleElement
    pub type TitleElementIter<R> = Iter<R, TitleElement, TeiReaderError, TitleElementIterFunction>;

    use super::AuthorElement;
    use super::AuthorElementIterFunction;
    /// Iterator for AuthorElement
    pub type AuthorElementIter<R> = Iter<R, AuthorElement, TeiReaderError, AuthorElementIterFunction>;

    use super::EditorElement;
    use super::EditorElementIterFunction;
    /// Iterator for EditorElement
    pub type EditorElementIter<R> = Iter<R, EditorElement, TeiReaderError, EditorElementIterFunction>;

    use super::RespStmtElement;
    use super::RespStmtElementIterFunction;
    /// Iterator for RespStmtElement
    pub type RespStmtElementIter<R> = Iter<R, RespStmtElement, TeiReaderError, RespStmtElementIterFunction>;

    use super::RespElement;
    use super::RespElementIterFunction;
    /// Iterator for RespElement
    pub type RespElementIter<R> = Iter<R, RespElement, TeiReaderError, RespElementIterFunction>;

    use super::NameElement;
    use super::NameElementIterFunction;
    /// Iterator for NameElement
    pub type NameElementIter<R> = Iter<R, NameElement, TeiReaderError, NameElementIterFunction>;

    use super::EditionStmtElement;
    use super::EditionStmtElementIterFunction;
    /// Iterator for EditionStmtElement
    pub type EditionStmtElementIter<R> = Iter<R, EditionStmtElement, TeiReaderError, EditionStmtElementIterFunction>;

    use super::EditionElement;
    use super::EditionElementIterFunction;
    /// Iterator for EditionElement
    pub type EditionElementIter<R> = Iter<R, EditionElement, TeiReaderError, EditionElementIterFunction>;

    use super::ExtentElement;
    use super::ExtentElementIterFunction;
    /// Iterator for ExtentElement
    pub type ExtentElementIter<R> = Iter<R, ExtentElement, TeiReaderError, ExtentElementIterFunction>;

    use super::PublicationStmtElement;
    use super::PublicationStmtElementIterFunction;
    /// Iterator for PublicationStmtElement
    pub type PublicationStmtElementIter<R> = Iter<R, PublicationStmtElement, TeiReaderError, PublicationStmtElementIterFunction>;

    use super::PublisherElement;
    use super::PublisherElementIterFunction;
    /// Iterator for PublisherElement
    pub type PublisherElementIter<R> = Iter<R, PublisherElement, TeiReaderError, PublisherElementIterFunction>;

    use super::AvailabilityElement;
    use super::AvailabilityElementIterFunction;
    /// Iterator for AvailabilityElement
    pub type AvailabilityElementIter<R> = Iter<R, AvailabilityElement, TeiReaderError, AvailabilityElementIterFunction>;

    use super::PElement;
    use super::PElementIterFunction;
    /// Iterator for PElement
    pub type PElementIter<R> = Iter<R, PElement, TeiReaderError, PElementIterFunction>;

    use super::RefElement;
    use super::RefElementIterFunction;
    /// Iterator for RefElement
    pub type RefElementIter<R> = Iter<R, RefElement, TeiReaderError, RefElementIterFunction>;

    use super::DateElement;
    use super::DateElementIterFunction;
    /// Iterator for DateElement
    pub type DateElementIter<R> = Iter<R, DateElement, TeiReaderError, DateElementIterFunction>;

    use super::PubPlaceElement;
    use super::PubPlaceElementIterFunction;
    /// Iterator for PubPlaceElement
    pub type PubPlaceElementIter<R> = Iter<R, PubPlaceElement, TeiReaderError, PubPlaceElementIterFunction>;

    use super::NotesStmtElement;
    use super::NotesStmtElementIterFunction;
    /// Iterator for NotesStmtElement
    pub type NotesStmtElementIter<R> = Iter<R, NotesStmtElement, TeiReaderError, NotesStmtElementIterFunction>;

    use super::NoteElement;
    use super::NoteElementIterFunction;
    /// Iterator for NoteElement
    pub type NoteElementIter<R> = Iter<R, NoteElement, TeiReaderError, NoteElementIterFunction>;

    use super::SourceDescElement;
    use super::SourceDescElementIterFunction;
    /// Iterator for SourceDescElement
    pub type SourceDescElementIter<R> = Iter<R, SourceDescElement, TeiReaderError, SourceDescElementIterFunction>;

    use super::PtrElement;
    use super::PtrElementIterFunction;
    /// Iterator for PtrElement
    pub type PtrElementIter<R> = Iter<R, PtrElement, TeiReaderError, PtrElementIterFunction>;

    use super::EncodingDescElement;
    use super::EncodingDescElementIterFunction;
    /// Iterator for EncodingDescElement
    pub type EncodingDescElementIter<R> = Iter<R, EncodingDescElement, TeiReaderError, EncodingDescElementIterFunction>;

    use super::ProjectDescElement;
    use super::ProjectDescElementIterFunction;
    /// Iterator for ProjectDescElement
    pub type ProjectDescElementIter<R> = Iter<R, ProjectDescElement, TeiReaderError, ProjectDescElementIterFunction>;

    use super::RevisionDescElement;
    use super::RevisionDescElementIterFunction;
    /// Iterator for RevisionDescElement
    pub type RevisionDescElementIter<R> = Iter<R, RevisionDescElement, TeiReaderError, RevisionDescElementIterFunction>;

    use super::ChangeElement;
    use super::ChangeElementIterFunction;
    /// Iterator for ChangeElement
    pub type ChangeElementIter<R> = Iter<R, ChangeElement, TeiReaderError, ChangeElementIterFunction>;

    use super::ListElement;
    use super::ListElementIterFunction;
    /// Iterator for ListElement
    pub type ListElementIter<R> = Iter<R, ListElement, TeiReaderError, ListElementIterFunction>;

    use super::HeadElement;
    use super::HeadElementIterFunction;
    /// Iterator for HeadElement
    pub type HeadElementIter<R> = Iter<R, HeadElement, TeiReaderError, HeadElementIterFunction>;

    use super::ItemElement;
    use super::ItemElementIterFunction;
    /// Iterator for ItemElement
    pub type ItemElementIter<R> = Iter<R, ItemElement, TeiReaderError, ItemElementIterFunction>;

    use super::TextElement;
    use super::TextElementIterFunction;
    /// Iterator for TextElement
    pub type TextElementIter<R> = Iter<R, TextElement, TeiReaderError, TextElementIterFunction>;

    use super::BodyElement;
    use super::BodyElementIterFunction;
    /// Iterator for BodyElement
    pub type BodyElementIter<R> = Iter<R, BodyElement, TeiReaderError, BodyElementIterFunction>;

    use super::EntryElement;
    use super::EntryElementIterFunction;
    /// Iterator for EntryElement
    pub type EntryElementIter<R> = Iter<R, EntryElement, TeiReaderError, EntryElementIterFunction>;

    use super::FormElement;
    use super::FormElementIterFunction;
    /// Iterator for FormElement
    pub type FormElementIter<R> = Iter<R, FormElement, TeiReaderError, FormElementIterFunction>;

    use super::OrthElement;
    use super::OrthElementIterFunction;
    /// Iterator for OrthElement
    pub type OrthElementIter<R> = Iter<R, OrthElement, TeiReaderError, OrthElementIterFunction>;

    use super::GramGrpElement;
    use super::GramGrpElementIterFunction;
    /// Iterator for GramGrpElement
    pub type GramGrpElementIter<R> = Iter<R, GramGrpElement, TeiReaderError, GramGrpElementIterFunction>;

    use super::GenElement;
    use super::GenElementIterFunction;
    /// Iterator for GenElement
    pub type GenElementIter<R> = Iter<R, GenElement, TeiReaderError, GenElementIterFunction>;

    use super::PosElement;
    use super::PosElementIterFunction;
    /// Iterator for PosElement
    pub type PosElementIter<R> = Iter<R, PosElement, TeiReaderError, PosElementIterFunction>;

    use super::NumberElement;
    use super::NumberElementIterFunction;
    /// Iterator for NumberElement
    pub type NumberElementIter<R> = Iter<R, NumberElement, TeiReaderError, NumberElementIterFunction>;

    use super::SenseElement;
    use super::SenseElementIterFunction;
    /// Iterator for SenseElement
    pub type SenseElementIter<R> = Iter<R, SenseElement, TeiReaderError, SenseElementIterFunction>;

    use super::CitElement;
    use super::CitElementIterFunction;
    /// Iterator for CitElement
    pub type CitElementIter<R> = Iter<R, CitElement, TeiReaderError, CitElementIterFunction>;

    use super::QuoteElement;
    use super::QuoteElementIterFunction;
    /// Iterator for QuoteElement
    pub type QuoteElementIter<R> = Iter<R, QuoteElement, TeiReaderError, QuoteElementIterFunction>;

    use super::XrElement;
    use super::XrElementIterFunction;
    /// Iterator for XrElement
    pub type XrElementIter<R> = Iter<R, XrElement, TeiReaderError, XrElementIterFunction>;

    use super::UsgElement;
    use super::UsgElementIterFunction;
    /// Iterator for UsgElement
    pub type UsgElementIter<R> = Iter<R, UsgElement, TeiReaderError, UsgElementIterFunction>;

    use super::SubcElement;
    use super::SubcElementIterFunction;
    /// Iterator for SubcElement
    pub type SubcElementIter<R> = Iter<R, SubcElement, TeiReaderError, SubcElementIterFunction>;

    use super::CollocElement;
    use super::CollocElementIterFunction;
    /// Iterator for CollocElement
    pub type CollocElementIter<R> = Iter<R, CollocElement, TeiReaderError, CollocElementIterFunction>;

    use super::TnsElement;
    use super::TnsElementIterFunction;
    /// Iterator for TnsElement
    pub type TnsElementIter<R> = Iter<R, TnsElement, TeiReaderError, TnsElementIterFunction>;

    use super::MoodElement;
    use super::MoodElementIterFunction;
    /// Iterator for MoodElement
    pub type MoodElementIter<R> = Iter<R, MoodElement, TeiReaderError, MoodElementIterFunction>;
}

#[derive(Debug, thiserror::Error)]
pub enum TeiReaderError{
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
    TeiElementBuilderError(#[from] TeiElementBuilderError),
    #[error(transparent)]
    TeiHeaderElementBuilderError(#[from] TeiHeaderElementBuilderError),
    #[error(transparent)]
    FileDescElementBuilderError(#[from] FileDescElementBuilderError),
    #[error(transparent)]
    TitleStmtElementBuilderError(#[from] TitleStmtElementBuilderError),
    #[error(transparent)]
    TitleElementBuilderError(#[from] TitleElementBuilderError),
    #[error(transparent)]
    AuthorElementBuilderError(#[from] AuthorElementBuilderError),
    #[error(transparent)]
    EditorElementBuilderError(#[from] EditorElementBuilderError),
    #[error(transparent)]
    RespStmtElementBuilderError(#[from] RespStmtElementBuilderError),
    #[error(transparent)]
    RespElementBuilderError(#[from] RespElementBuilderError),
    #[error(transparent)]
    NameElementBuilderError(#[from] NameElementBuilderError),
    #[error(transparent)]
    EditionStmtElementBuilderError(#[from] EditionStmtElementBuilderError),
    #[error(transparent)]
    EditionElementBuilderError(#[from] EditionElementBuilderError),
    #[error(transparent)]
    ExtentElementBuilderError(#[from] ExtentElementBuilderError),
    #[error(transparent)]
    PublicationStmtElementBuilderError(#[from] PublicationStmtElementBuilderError),
    #[error(transparent)]
    PublisherElementBuilderError(#[from] PublisherElementBuilderError),
    #[error(transparent)]
    AvailabilityElementBuilderError(#[from] AvailabilityElementBuilderError),
    #[error(transparent)]
    PElementBuilderError(#[from] PElementBuilderError),
    #[error(transparent)]
    RefElementBuilderError(#[from] RefElementBuilderError),
    #[error(transparent)]
    DateElementBuilderError(#[from] DateElementBuilderError),
    #[error(transparent)]
    PubPlaceElementBuilderError(#[from] PubPlaceElementBuilderError),
    #[error(transparent)]
    NotesStmtElementBuilderError(#[from] NotesStmtElementBuilderError),
    #[error(transparent)]
    NoteElementBuilderError(#[from] NoteElementBuilderError),
    #[error(transparent)]
    SourceDescElementBuilderError(#[from] SourceDescElementBuilderError),
    #[error(transparent)]
    PtrElementBuilderError(#[from] PtrElementBuilderError),
    #[error(transparent)]
    EncodingDescElementBuilderError(#[from] EncodingDescElementBuilderError),
    #[error(transparent)]
    ProjectDescElementBuilderError(#[from] ProjectDescElementBuilderError),
    #[error(transparent)]
    RevisionDescElementBuilderError(#[from] RevisionDescElementBuilderError),
    #[error(transparent)]
    ChangeElementBuilderError(#[from] ChangeElementBuilderError),
    #[error(transparent)]
    ListElementBuilderError(#[from] ListElementBuilderError),
    #[error(transparent)]
    HeadElementBuilderError(#[from] HeadElementBuilderError),
    #[error(transparent)]
    ItemElementBuilderError(#[from] ItemElementBuilderError),
    #[error(transparent)]
    TextElementBuilderError(#[from] TextElementBuilderError),
    #[error(transparent)]
    BodyElementBuilderError(#[from] BodyElementBuilderError),
    #[error(transparent)]
    EntryElementBuilderError(#[from] EntryElementBuilderError),
    #[error(transparent)]
    FormElementBuilderError(#[from] FormElementBuilderError),
    #[error(transparent)]
    OrthElementBuilderError(#[from] OrthElementBuilderError),
    #[error(transparent)]
    GramGrpElementBuilderError(#[from] GramGrpElementBuilderError),
    #[error(transparent)]
    GenElementBuilderError(#[from] GenElementBuilderError),
    #[error(transparent)]
    PosElementBuilderError(#[from] PosElementBuilderError),
    #[error(transparent)]
    NumberElementBuilderError(#[from] NumberElementBuilderError),
    #[error(transparent)]
    SenseElementBuilderError(#[from] SenseElementBuilderError),
    #[error(transparent)]
    CitElementBuilderError(#[from] CitElementBuilderError),
    #[error(transparent)]
    QuoteElementBuilderError(#[from] QuoteElementBuilderError),
    #[error(transparent)]
    XrElementBuilderError(#[from] XrElementBuilderError),
    #[error(transparent)]
    UsgElementBuilderError(#[from] UsgElementBuilderError),
    #[error(transparent)]
    SubcElementBuilderError(#[from] SubcElementBuilderError),
    #[error(transparent)]
    CollocElementBuilderError(#[from] CollocElementBuilderError),
    #[error(transparent)]
    TnsElementBuilderError(#[from] TnsElementBuilderError),
    #[error(transparent)]
    MoodElementBuilderError(#[from] MoodElementBuilderError),
}