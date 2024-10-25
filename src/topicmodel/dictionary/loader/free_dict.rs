#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test11(){
        let x = BufReader::new(File::open(r#"D:\Downloads\freedict-eng-deu-1.9-fd1.src\eng-deu\eng-deu.tei"#).unwrap());
        match super::read_as_root_tei_element(&mut quick_xml::reader::Reader::from_reader(x)) {
            Ok(value) => {
                assert!(value.is_some());
            }
            Err(error) => {
                println!("{error}")
            }
        }
    }
    #[test]
    fn test12(){
        let x = BufReader::new(File::open(r#"D:\Downloads\freedict-deu-eng-1.9-fd1.src\deu-eng\deu-eng.tei"#).unwrap());
        match super::read_as_root_tei_element(&mut quick_xml::reader::Reader::from_reader(x)) {
            Ok(value) => {
                assert!(value.is_some());
            }
            Err(error) => {
                println!("{error}")
            }
        }
    }

    #[test]
    fn test21(){
        let x = BufReader::new(File::open(r#"D:\Downloads\freedict-eng-deu-1.9-fd1.src\eng-deu\eng-deu.tei"#).unwrap());
        let reader = quick_xml::reader::Reader::from_reader(x);
        let mut i = super::iter_for_entry_element(reader);
        let mut x = 0;
        while let Some(value) = i.next() {
            match value {
                Ok(value) => {
                    x+=1usize;
                }
                Err(error) => {
                    println!("ERROR: {error}");
                    break
                }
            }
        }
        println!("{x}");
    }

    #[test]
    fn test22(){
        let x = BufReader::new(File::open(r#"D:\Downloads\freedict-deu-eng-1.9-fd1.src\deu-eng\deu-eng.tei"#).unwrap());
        let reader = quick_xml::reader::Reader::from_reader(x);
        let mut i = super::iter_for_entry_element(reader);
        let mut x = 0;
        while let Some(value) = i.next() {
            match value {
                Ok(value) => {
                    x+=1usize;
                }
                Err(error) => {
                    println!("ERROR: {error}");
                    break
                }
            }
        }
        println!("{x}");
    }
}


// Element - TEI - TEI
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TeiElement {
    #[builder(setter(strip_option), default)]
    pub xmlns_attribute: Option<String>,
    #[builder(setter(strip_option), default)]
    pub version_attribute: Option<f64>,
    #[builder(setter(custom), default)]
    pub tei_header_element: Vec<TeiHeaderElement>,
    #[builder(setter(custom), default)]
    pub text_element: Vec<TextElement>,
}

impl TeiElementBuilder{
    pub fn tei_header_element(&mut self, value: TeiHeaderElement){
        let targ = self.tei_header_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn text_element(&mut self, value: TextElement){
        let targ = self.text_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_tei_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, TeiElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<TeiElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_tei_element(r);
    iter::Iter::new(reader, f)
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
    pub encoding_desc_element: Vec<EncodingDescElement>,
    #[builder(setter(custom), default)]
    pub file_desc_element: Vec<FileDescElement>,
    #[builder(setter(custom), default)]
    pub revision_desc_element: Vec<RevisionDescElement>,
}

impl TeiHeaderElementBuilder{
    pub fn encoding_desc_element(&mut self, value: EncodingDescElement){
        let targ = self.encoding_desc_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn file_desc_element(&mut self, value: FileDescElement){
        let targ = self.file_desc_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn revision_desc_element(&mut self, value: RevisionDescElement){
        let targ = self.revision_desc_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_tei_header_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, TeiHeaderElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<TeiHeaderElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_tei_header_element(r);
    iter::Iter::new(reader, f)
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
                    b"encodingDesc" => {
                        let recognized = read_encoding_desc_element(reader, start)?;
                        builder.encoding_desc_element(recognized);
                    }
                    b"fileDesc" => {
                        let recognized = read_file_desc_element(reader, start)?;
                        builder.file_desc_element(recognized);
                    }
                    b"revisionDesc" => {
                        let recognized = read_revision_desc_element(reader, start)?;
                        builder.revision_desc_element(recognized);
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
                    b"encodingDesc" => {
                        let recognized = read_encoding_desc_element(reader, value)?;
                        builder.encoding_desc_element(recognized);
                    }
                    b"fileDesc" => {
                        let recognized = read_file_desc_element(reader, value)?;
                        builder.file_desc_element(recognized);
                    }
                    b"revisionDesc" => {
                        let recognized = read_revision_desc_element(reader, value)?;
                        builder.revision_desc_element(recognized);
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
    pub extent_element: Vec<ExtentElement>,
    #[builder(setter(custom), default)]
    pub edition_stmt_element: Vec<EditionStmtElement>,
    #[builder(setter(custom), default)]
    pub notes_stmt_element: Vec<NotesStmtElement>,
    #[builder(setter(custom), default)]
    pub publication_stmt_element: Vec<PublicationStmtElement>,
    #[builder(setter(custom), default)]
    pub source_desc_element: Vec<SourceDescElement>,
    #[builder(setter(custom), default)]
    pub title_stmt_element: Vec<TitleStmtElement>,
}

impl FileDescElementBuilder{
    pub fn extent_element(&mut self, value: ExtentElement){
        let targ = self.extent_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn edition_stmt_element(&mut self, value: EditionStmtElement){
        let targ = self.edition_stmt_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn notes_stmt_element(&mut self, value: NotesStmtElement){
        let targ = self.notes_stmt_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn publication_stmt_element(&mut self, value: PublicationStmtElement){
        let targ = self.publication_stmt_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn source_desc_element(&mut self, value: SourceDescElement){
        let targ = self.source_desc_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn title_stmt_element(&mut self, value: TitleStmtElement){
        let targ = self.title_stmt_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_file_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, FileDescElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<FileDescElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_file_desc_element(r);
    iter::Iter::new(reader, f)
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
    }
}

pub fn read_file_desc_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<FileDescElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = FileDescElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"extent" => {
                        let recognized = read_extent_element(reader, start)?;
                        builder.extent_element(recognized);
                    }
                    b"editionStmt" => {
                        let recognized = read_edition_stmt_element(reader, start)?;
                        builder.edition_stmt_element(recognized);
                    }
                    b"notesStmt" => {
                        let recognized = read_notes_stmt_element(reader, start)?;
                        builder.notes_stmt_element(recognized);
                    }
                    b"publicationStmt" => {
                        let recognized = read_publication_stmt_element(reader, start)?;
                        builder.publication_stmt_element(recognized);
                    }
                    b"sourceDesc" => {
                        let recognized = read_source_desc_element(reader, start)?;
                        builder.source_desc_element(recognized);
                    }
                    b"titleStmt" => {
                        let recognized = read_title_stmt_element(reader, start)?;
                        builder.title_stmt_element(recognized);
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
                    b"extent" => {
                        let recognized = read_extent_element(reader, value)?;
                        builder.extent_element(recognized);
                    }
                    b"editionStmt" => {
                        let recognized = read_edition_stmt_element(reader, value)?;
                        builder.edition_stmt_element(recognized);
                    }
                    b"notesStmt" => {
                        let recognized = read_notes_stmt_element(reader, value)?;
                        builder.notes_stmt_element(recognized);
                    }
                    b"publicationStmt" => {
                        let recognized = read_publication_stmt_element(reader, value)?;
                        builder.publication_stmt_element(recognized);
                    }
                    b"sourceDesc" => {
                        let recognized = read_source_desc_element(reader, value)?;
                        builder.source_desc_element(recognized);
                    }
                    b"titleStmt" => {
                        let recognized = read_title_stmt_element(reader, value)?;
                        builder.title_stmt_element(recognized);
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
    pub title_element: Vec<TitleElement>,
    #[builder(setter(custom), default)]
    pub author_element: Vec<AuthorElement>,
    #[builder(setter(custom), default)]
    pub editor_element: Vec<EditorElement>,
    #[builder(setter(custom), default)]
    pub resp_stmt_element: Vec<RespStmtElement>,
}

impl TitleStmtElementBuilder{
    pub fn title_element(&mut self, value: TitleElement){
        let targ = self.title_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn author_element(&mut self, value: AuthorElement){
        let targ = self.author_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn editor_element(&mut self, value: EditorElement){
        let targ = self.editor_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn resp_stmt_element(&mut self, value: RespStmtElement){
        let targ = self.resp_stmt_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_title_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, TitleStmtElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<TitleStmtElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_title_stmt_element(r);
    iter::Iter::new(reader, f)
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
    }
}

pub fn read_title_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<TitleStmtElement, TeiReaderError>{
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
                    b"author" => {
                        let recognized = read_author_element(reader, start)?;
                        builder.author_element(recognized);
                    }
                    b"editor" => {
                        let recognized = read_editor_element(reader, start)?;
                        builder.editor_element(recognized);
                    }
                    b"respStmt" => {
                        let recognized = read_resp_stmt_element(reader, start)?;
                        builder.resp_stmt_element(recognized);
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
                    b"author" => {
                        let recognized = read_author_element(reader, value)?;
                        builder.author_element(recognized);
                    }
                    b"editor" => {
                        let recognized = read_editor_element(reader, value)?;
                        builder.editor_element(recognized);
                    }
                    b"respStmt" => {
                        let recognized = read_resp_stmt_element(reader, value)?;
                        builder.resp_stmt_element(recognized);
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_title_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, TitleElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<TitleElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_title_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_author_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, AuthorElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<AuthorElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_author_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_editor_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, EditorElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<EditorElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_editor_element(r);
    iter::Iter::new(reader, f)
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
    pub name_element: Vec<NameElement>,
    #[builder(setter(custom), default)]
    pub resp_element: Vec<RespElement>,
}

impl RespStmtElementBuilder{
    pub fn name_element(&mut self, value: NameElement){
        let targ = self.name_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn resp_element(&mut self, value: RespElement){
        let targ = self.resp_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_resp_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, RespStmtElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<RespStmtElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_resp_stmt_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<ERespElement>,
}

pub fn iter_for_resp_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, RespElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<RespElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_resp_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_name_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, NameElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<NameElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_name_element(r);
    iter::Iter::new(reader, f)
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
    pub edition_element: Vec<EditionElement>,
}

impl EditionStmtElementBuilder{
    pub fn edition_element(&mut self, value: EditionElement){
        let targ = self.edition_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_edition_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, EditionStmtElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<EditionStmtElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_edition_stmt_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_edition_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, EditionElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<EditionElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_edition_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_extent_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, ExtentElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<ExtentElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_extent_element(r);
    iter::Iter::new(reader, f)
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
    pub availability_element: Vec<AvailabilityElement>,
    #[builder(setter(custom), default)]
    pub publisher_element: Vec<PublisherElement>,
    #[builder(setter(custom), default)]
    pub pub_place_element: Vec<PubPlaceElement>,
    #[builder(setter(custom), default)]
    pub date_element: Vec<DateElement>,
}

impl PublicationStmtElementBuilder{
    pub fn availability_element(&mut self, value: AvailabilityElement){
        let targ = self.availability_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn publisher_element(&mut self, value: PublisherElement){
        let targ = self.publisher_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn pub_place_element(&mut self, value: PubPlaceElement){
        let targ = self.pub_place_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn date_element(&mut self, value: DateElement){
        let targ = self.date_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_publication_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, PublicationStmtElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<PublicationStmtElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_publication_stmt_element(r);
    iter::Iter::new(reader, f)
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
    }
}

pub fn read_publication_stmt_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<PublicationStmtElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = PublicationStmtElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"availability" => {
                        let recognized = read_availability_element(reader, start)?;
                        builder.availability_element(recognized);
                    }
                    b"publisher" => {
                        let recognized = read_publisher_element(reader, start)?;
                        builder.publisher_element(recognized);
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
                    b"availability" => {
                        let recognized = read_availability_element(reader, value)?;
                        builder.availability_element(recognized);
                    }
                    b"publisher" => {
                        let recognized = read_publisher_element(reader, value)?;
                        builder.publisher_element(recognized);
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
    #[builder(setter(strip_option), default)]
    pub content: Option<EPublisherElement>,
}

pub fn iter_for_publisher_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, PublisherElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<PublisherElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_publisher_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub status_attribute: Option<StatusAttribute>,
    #[builder(setter(custom), default)]
    pub p_element: Vec<PElement>,
}

impl AvailabilityElementBuilder{
    pub fn p_element(&mut self, value: PElement){
        let targ = self.p_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_availability_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, AvailabilityElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<AvailabilityElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_availability_element(r);
    iter::Iter::new(reader, f)
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
    pub ref_element: Vec<RefElement>,
    #[builder(setter(custom), default)]
    pub ptr_element: Vec<PtrElement>,
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

impl PElementBuilder{
    pub fn ref_element(&mut self, value: RefElement){
        let targ = self.ref_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn ptr_element(&mut self, value: PtrElement){
        let targ = self.ptr_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_p_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, PElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<PElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_p_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_ref_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, RefElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<RefElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_ref_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub when_attribute: Option<String>,
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_date_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, DateElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<DateElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_date_element(r);
    iter::Iter::new(reader, f)
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
    pub ref_element: Vec<RefElement>,
}

impl PubPlaceElementBuilder{
    pub fn ref_element(&mut self, value: RefElement){
        let targ = self.ref_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_pub_place_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, PubPlaceElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<PubPlaceElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_pub_place_element(r);
    iter::Iter::new(reader, f)
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
    pub note_element: Vec<NoteElement>,
}

impl NotesStmtElementBuilder{
    pub fn note_element(&mut self, value: NoteElement){
        let targ = self.note_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_notes_stmt_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, NotesStmtElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<NotesStmtElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_notes_stmt_element(r);
    iter::Iter::new(reader, f)
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

pub fn iter_for_note_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, NoteElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<NoteElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_note_element(r);
    iter::Iter::new(reader, f)
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
    pub p_element: Vec<PElement>,
}

impl SourceDescElementBuilder{
    pub fn p_element(&mut self, value: PElement){
        let targ = self.p_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_source_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, SourceDescElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<SourceDescElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_source_desc_element(r);
    iter::Iter::new(reader, f)
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

pub fn iter_for_ptr_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, PtrElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<PtrElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_ptr_element(r);
    iter::Iter::new(reader, f)
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
    pub project_desc_element: Vec<ProjectDescElement>,
}

impl EncodingDescElementBuilder{
    pub fn project_desc_element(&mut self, value: ProjectDescElement){
        let targ = self.project_desc_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_encoding_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, EncodingDescElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<EncodingDescElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_encoding_desc_element(r);
    iter::Iter::new(reader, f)
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
    pub p_element: Vec<PElement>,
}

impl ProjectDescElementBuilder{
    pub fn p_element(&mut self, value: PElement){
        let targ = self.p_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_project_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, ProjectDescElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<ProjectDescElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_project_desc_element(r);
    iter::Iter::new(reader, f)
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
    pub change_element: Vec<ChangeElement>,
}

impl RevisionDescElementBuilder{
    pub fn change_element(&mut self, value: ChangeElement){
        let targ = self.change_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_revision_desc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, RevisionDescElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<RevisionDescElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_revision_desc_element(r);
    iter::Iter::new(reader, f)
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
    pub n_attribute: Option<String>,
    #[builder(setter(strip_option), default)]
    pub when_attribute: Option<String>,
    #[builder(setter(strip_option), default)]
    pub who_attribute: Option<String>,
    #[builder(setter(custom), default)]
    pub list_element: Vec<ListElement>,
}

impl ChangeElementBuilder{
    pub fn list_element(&mut self, value: ListElement){
        let targ = self.list_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_change_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, ChangeElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<ChangeElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_change_element(r);
    iter::Iter::new(reader, f)
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
    }
}

pub fn read_change_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, start: quick_xml::events::BytesStart<'a>) -> Result<ChangeElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = ChangeElementBuilder::create_empty();
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if let Some(value) = read_n_attribute(&attr)? {
                    builder.n_attribute(value);
                    continue;
                }
                if let Some(value) = read_when_attribute(&attr)? {
                    builder.when_attribute(value);
                    continue;
                }
                if let Some(value) = read_who_attribute(&attr)? {
                    builder.who_attribute(value);
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
    pub head_element: Vec<HeadElement>,
    #[builder(setter(custom), default)]
    pub item_element: Vec<ItemElement>,
}

impl ListElementBuilder{
    pub fn head_element(&mut self, value: HeadElement){
        let targ = self.head_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn item_element(&mut self, value: ItemElement){
        let targ = self.item_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_list_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, ListElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<ListElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_list_element(r);
    iter::Iter::new(reader, f)
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
                    b"head" => {
                        let recognized = read_head_element(reader, start)?;
                        builder.head_element(recognized);
                    }
                    b"item" => {
                        let recognized = read_item_element(reader, start)?;
                        builder.item_element(recognized);
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
                    b"head" => {
                        let recognized = read_head_element(reader, value)?;
                        builder.head_element(recognized);
                    }
                    b"item" => {
                        let recognized = read_item_element(reader, value)?;
                        builder.item_element(recognized);
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
    pub date_element: Vec<DateElement>,
    #[builder(setter(custom), default)]
    pub name_element: Vec<NameElement>,
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

impl HeadElementBuilder{
    pub fn date_element(&mut self, value: DateElement){
        let targ = self.date_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn name_element(&mut self, value: NameElement){
        let targ = self.name_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_head_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, HeadElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<HeadElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_head_element(r);
    iter::Iter::new(reader, f)
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
    }
}

pub fn read_head_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<HeadElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = HeadElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"date" => {
                        let recognized = read_date_element(reader, start)?;
                        builder.date_element(recognized);
                    }
                    b"name" => {
                        let recognized = read_name_element(reader, start)?;
                        builder.name_element(recognized);
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
                    b"date" => {
                        let recognized = read_date_element(reader, value)?;
                        builder.date_element(recognized);
                    }
                    b"name" => {
                        let recognized = read_name_element(reader, value)?;
                        builder.name_element(recognized);
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_item_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, ItemElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<ItemElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_item_element(r);
    iter::Iter::new(reader, f)
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
    pub body_element: Vec<BodyElement>,
}

impl TextElementBuilder{
    pub fn body_element(&mut self, value: BodyElement){
        let targ = self.body_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_text_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, TextElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<TextElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_text_element(r);
    iter::Iter::new(reader, f)
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
    pub entry_element: Vec<EntryElement>,
}

impl BodyElementBuilder{
    pub fn entry_element(&mut self, value: EntryElement){
        let targ = self.entry_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_body_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, BodyElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<BodyElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_body_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub id_attribute: Option<String>,
    #[builder(setter(custom), default)]
    pub sense_element: Vec<SenseElement>,
    #[builder(setter(custom), default)]
    pub form_element: Vec<FormElement>,
    #[builder(setter(custom), default)]
    pub gram_grp_element: Vec<GramGrpElement>,
}

impl EntryElementBuilder{
    pub fn sense_element(&mut self, value: SenseElement){
        let targ = self.sense_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn form_element(&mut self, value: FormElement){
        let targ = self.form_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn gram_grp_element(&mut self, value: GramGrpElement){
        let targ = self.gram_grp_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_entry_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, EntryElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<EntryElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_entry_element(r);
    iter::Iter::new(reader, f)
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
                    b"form" => {
                        let recognized = read_form_element(reader, start)?;
                        builder.form_element(recognized);
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
                    b"form" => {
                        let recognized = read_form_element(reader, value)?;
                        builder.form_element(recognized);
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

// Element - form - form
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct FormElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(custom), default)]
    pub gram_grp_element: Vec<GramGrpElement>,
    #[builder(setter(custom), default)]
    pub usg_element: Vec<UsgElement>,
    #[builder(setter(custom), default)]
    pub orth_element: Vec<OrthElement>,
    #[builder(setter(custom), default)]
    pub form_element: Vec<FormElement>,
}

impl FormElementBuilder{
    pub fn gram_grp_element(&mut self, value: GramGrpElement){
        let targ = self.gram_grp_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn usg_element(&mut self, value: UsgElement){
        let targ = self.usg_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn orth_element(&mut self, value: OrthElement){
        let targ = self.orth_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn form_element(&mut self, value: FormElement){
        let targ = self.form_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_form_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, FormElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<FormElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_form_element(r);
    iter::Iter::new(reader, f)
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
                    b"usg" => {
                        let recognized = read_usg_element(reader, start)?;
                        builder.usg_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, start)?;
                        builder.orth_element(recognized);
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
                    b"usg" => {
                        let recognized = read_usg_element(reader, value)?;
                        builder.usg_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, value)?;
                        builder.orth_element(recognized);
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_orth_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, OrthElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<OrthElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_orth_element(r);
    iter::Iter::new(reader, f)
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

// Element - sense - sense
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct SenseElement {
    #[builder(setter(custom), default)]
    pub cit_element: Vec<CitElement>,
    #[builder(setter(custom), default)]
    pub xr_element: Vec<XrElement>,
    #[builder(setter(custom), default)]
    pub note_element: Vec<NoteElement>,
    #[builder(setter(custom), default)]
    pub usg_element: Vec<UsgElement>,
}

impl SenseElementBuilder{
    pub fn cit_element(&mut self, value: CitElement){
        let targ = self.cit_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn xr_element(&mut self, value: XrElement){
        let targ = self.xr_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn note_element(&mut self, value: NoteElement){
        let targ = self.note_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn usg_element(&mut self, value: UsgElement){
        let targ = self.usg_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_sense_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, SenseElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<SenseElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_sense_element(r);
    iter::Iter::new(reader, f)
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
    }
}

pub fn read_sense_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<SenseElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = SenseElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"cit" => {
                        let recognized = read_cit_element(reader, start)?;
                        builder.cit_element(recognized);
                    }
                    b"xr" => {
                        let recognized = read_xr_element(reader, start)?;
                        builder.xr_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, start)?;
                        builder.note_element(recognized);
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
                    b"sense" => {
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
                    b"xr" => {
                        let recognized = read_xr_element(reader, value)?;
                        builder.xr_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, value)?;
                        builder.note_element(recognized);
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

// Element - cit - cit
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct CitElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(custom), default)]
    pub quote_element: Vec<QuoteElement>,
    #[builder(setter(custom), default)]
    pub gram_grp_element: Vec<GramGrpElement>,
    #[builder(setter(custom), default)]
    pub note_element: Vec<NoteElement>,
    #[builder(setter(custom), default)]
    pub usg_element: Vec<UsgElement>,
    #[builder(setter(custom), default)]
    pub orth_element: Vec<OrthElement>,
    #[builder(setter(custom), default)]
    pub cit_element: Vec<CitElement>,
}

impl CitElementBuilder{
    pub fn quote_element(&mut self, value: QuoteElement){
        let targ = self.quote_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn gram_grp_element(&mut self, value: GramGrpElement){
        let targ = self.gram_grp_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn note_element(&mut self, value: NoteElement){
        let targ = self.note_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn usg_element(&mut self, value: UsgElement){
        let targ = self.usg_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn orth_element(&mut self, value: OrthElement){
        let targ = self.orth_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn cit_element(&mut self, value: CitElement){
        let targ = self.cit_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_cit_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, CitElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<CitElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_cit_element(r);
    iter::Iter::new(reader, f)
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
                    b"quote" => {
                        let recognized = read_quote_element(reader, start)?;
                        builder.quote_element(recognized);
                    }
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, start)?;
                        builder.gram_grp_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, start)?;
                        builder.note_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, start)?;
                        builder.usg_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, start)?;
                        builder.orth_element(recognized);
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
                    b"quote" => {
                        let recognized = read_quote_element(reader, value)?;
                        builder.quote_element(recognized);
                    }
                    b"gramGrp" => {
                        let recognized = read_gram_grp_element(reader, value)?;
                        builder.gram_grp_element(recognized);
                    }
                    b"note" => {
                        let recognized = read_note_element(reader, value)?;
                        builder.note_element(recognized);
                    }
                    b"usg" => {
                        let recognized = read_usg_element(reader, value)?;
                        builder.usg_element(recognized);
                    }
                    b"orth" => {
                        let recognized = read_orth_element(reader, value)?;
                        builder.orth_element(recognized);
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_quote_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, QuoteElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<QuoteElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_quote_element(r);
    iter::Iter::new(reader, f)
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

// Element - gramGrp - gramGrp
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct GramGrpElement {
    #[builder(setter(custom), default)]
    pub pos_element: Vec<PosElement>,
    #[builder(setter(custom), default)]
    pub number_element: Vec<NumberElement>,
    #[builder(setter(custom), default)]
    pub colloc_element: Vec<CollocElement>,
    #[builder(setter(custom), default)]
    pub mood_element: Vec<MoodElement>,
    #[builder(setter(custom), default)]
    pub tns_element: Vec<TnsElement>,
    #[builder(setter(custom), default)]
    pub gen_element: Vec<GenElement>,
    #[builder(setter(custom), default)]
    pub subc_element: Vec<SubcElement>,
}

impl GramGrpElementBuilder{
    pub fn pos_element(&mut self, value: PosElement){
        let targ = self.pos_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn number_element(&mut self, value: NumberElement){
        let targ = self.number_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn colloc_element(&mut self, value: CollocElement){
        let targ = self.colloc_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn mood_element(&mut self, value: MoodElement){
        let targ = self.mood_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn tns_element(&mut self, value: TnsElement){
        let targ = self.tns_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn gen_element(&mut self, value: GenElement){
        let targ = self.gen_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
    pub fn subc_element(&mut self, value: SubcElement){
        let targ = self.subc_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_gram_grp_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, GramGrpElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<GramGrpElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_gram_grp_element(r);
    iter::Iter::new(reader, f)
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
    }
}

pub fn read_gram_grp_element<'a, R: std::io::BufRead>(reader: &mut quick_xml::reader::Reader<R>, _start: quick_xml::events::BytesStart<'a>) -> Result<GramGrpElement, TeiReaderError>{
    let mut buffer = Vec::new();
    let mut builder = GramGrpElementBuilder::create_empty();
    loop{
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(start) => {
                match start.local_name().as_ref(){
                    b"pos" => {
                        let recognized = read_pos_element(reader, start)?;
                        builder.pos_element(recognized);
                    }
                    b"number" => {
                        let recognized = read_number_element(reader, start)?;
                        builder.number_element(recognized);
                    }
                    b"colloc" => {
                        let recognized = read_colloc_element(reader, start)?;
                        builder.colloc_element(recognized);
                    }
                    b"mood" => {
                        let recognized = read_mood_element(reader, start)?;
                        builder.mood_element(recognized);
                    }
                    b"tns" => {
                        let recognized = read_tns_element(reader, start)?;
                        builder.tns_element(recognized);
                    }
                    b"gen" => {
                        let recognized = read_gen_element(reader, start)?;
                        builder.gen_element(recognized);
                    }
                    b"subc" => {
                        let recognized = read_subc_element(reader, start)?;
                        builder.subc_element(recognized);
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
                    b"pos" => {
                        let recognized = read_pos_element(reader, value)?;
                        builder.pos_element(recognized);
                    }
                    b"number" => {
                        let recognized = read_number_element(reader, value)?;
                        builder.number_element(recognized);
                    }
                    b"colloc" => {
                        let recognized = read_colloc_element(reader, value)?;
                        builder.colloc_element(recognized);
                    }
                    b"mood" => {
                        let recognized = read_mood_element(reader, value)?;
                        builder.mood_element(recognized);
                    }
                    b"tns" => {
                        let recognized = read_tns_element(reader, value)?;
                        builder.tns_element(recognized);
                    }
                    b"gen" => {
                        let recognized = read_gen_element(reader, value)?;
                        builder.gen_element(recognized);
                    }
                    b"subc" => {
                        let recognized = read_subc_element(reader, value)?;
                        builder.subc_element(recognized);
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
    #[builder(setter(strip_option), default)]
    pub content: Option<EGenElement>,
}

pub fn iter_for_gen_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, GenElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<GenElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_gen_element(r);
    iter::Iter::new(reader, f)
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

// Element - usg - usg
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct UsgElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_usg_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, UsgElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<UsgElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_usg_element(r);
    iter::Iter::new(reader, f)
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

// Element - xr - xr
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct XrElement {
    #[builder(setter(strip_option), default)]
    pub type_attribute: Option<TypeAttribute>,
    #[builder(setter(custom), default)]
    pub ref_element: Vec<RefElement>,
}

impl XrElementBuilder{
    pub fn ref_element(&mut self, value: RefElement){
        let targ = self.ref_element.get_or_insert_with(Default::default);
        targ.push(value);
    }
}
pub fn iter_for_xr_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, XrElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<XrElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_xr_element(r);
    iter::Iter::new(reader, f)
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum ENumberElement {
    #[strum(serialize="pl")]
    Pl,
    #[strum(serialize="sg")]
    Sg,
}
// Element - number - number
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct NumberElement {
    #[builder(setter(strip_option), default)]
    pub content: Option<ENumberElement>,
}

pub fn iter_for_number_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, NumberElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<NumberElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_number_element(r);
    iter::Iter::new(reader, f)
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum EPosElement {
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
    #[strum(serialize="n")]
    N,
}
// Element - pos - pos
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PosElement {
    #[builder(setter(strip_option), default)]
    pub content: Option<EPosElement>,
}

pub fn iter_for_pos_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, PosElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<PosElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_pos_element(r);
    iter::Iter::new(reader, f)
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

// Element - subc - subc
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct SubcElement {
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_subc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, SubcElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<SubcElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_subc_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<String>,
}

pub fn iter_for_colloc_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, CollocElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<CollocElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_colloc_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<ETnsElement>,
}

pub fn iter_for_tns_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, TnsElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<TnsElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_tns_element(r);
    iter::Iter::new(reader, f)
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
    #[builder(setter(strip_option), default)]
    pub content: Option<EMoodElement>,
}

pub fn iter_for_mood_element<R: std::io::BufRead>(reader: quick_xml::reader::Reader<R>) -> iter::Iter<R, MoodElement, TeiReaderError, impl for<'a> Fn(&'a mut quick_xml::reader::Reader<R>) -> Result<Option<MoodElement>, TeiReaderError>>{
    let f = |r: &mut quick_xml::reader::Reader<R>| read_as_root_mood_element(r);
    iter::Iter::new(reader, f)
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
        Ok(Some(value.into_owned()))
    } else { Ok(None) }
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
    #[strum(serialize="en")]
    En,
    #[strum(serialize="de")]
    De,
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
        Ok(Some(value.into_owned()))
    } else { Ok(None) }
}
// Attribute - when - WhenAttribute
pub fn read_when_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"when" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))
    } else { Ok(None) }
}
// Attribute - type - TypeAttribute
#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display, strum::EnumString)]
pub enum TypeAttribute {
    #[strum(serialize="trans")]
    Trans,
    #[strum(serialize="status")]
    Status,
    #[strum(serialize="syn")]
    Syn,
    #[strum(serialize="reg")]
    Reg,
    #[strum(serialize="style")]
    Style,
    #[strum(serialize="dom")]
    Dom,
    #[strum(serialize="time")]
    Time,
    #[strum(serialize="hint")]
    Hint,
    #[strum(serialize="lang")]
    Lang,
    #[strum(serialize="bulleted")]
    Bulleted,
    #[strum(serialize="abbrev")]
    Abbrev,
    #[strum(serialize="infl")]
    Infl,
    #[strum(serialize="see")]
    See,
    #[strum(serialize="geo")]
    Geo,
    #[strum(serialize="example")]
    Example,
    #[strum(serialize="colloc")]
    Colloc,
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
        Ok(Some(value.into_owned()))
    } else { Ok(None) }
}
// Attribute - who - WhoAttribute
pub fn read_who_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"who" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))
    } else { Ok(None) }
}
// Attribute - id - IdAttribute
pub fn read_id_attribute(attr: &quick_xml::events::attributes::Attribute) -> Result<Option<String>, TeiReaderError>{
    if attr.key.local_name().as_ref() == b"id" {
        let value = attr.unescape_value()?;
        Ok(Some(value.into_owned()))
    } else { Ok(None) }
}

mod iter {
    trait IterHelper<R, I, E> {
        fn goto_next(&self, reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<I>, E>;
    }

    impl<R, I, E, F> IterHelper<R, I, E> for F
    where
        F: Fn(&mut quick_xml::reader::Reader<R>) -> Result<Option<I>, E>,
        R: std::io::BufRead,
        I: Sized,
        E: std::error::Error
    {
        #[inline(always)]
        fn goto_next(&self, reader: &mut quick_xml::reader::Reader<R>) -> Result<Option<I>, E> {
            self(reader)
        }
    }

    pub struct Iter<R, I, E, H> where H: IterHelper<R, I, E> {
        reader: quick_xml::reader::Reader<R>,
        read_method: H,
        _phantom: std::marker::PhantomData<(E, I)>
    }

    impl<R, I, E, H> Iter<R, I, E, H>
    where
        H: IterHelper<R, I, E>
    {
        pub(super) fn new(reader: quick_xml::reader::Reader<R>, read_method: H) -> Self {
            Self { reader, read_method, _phantom: std::marker::PhantomData }
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
        H: IterHelper<R, I, E>
    {
        type Item = Result<I, E>;

        fn next(&mut self) -> Option<Self::Item> {
            self.read_method.goto_next(&mut self.reader).transpose()
        }
    }
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
    SenseElementBuilderError(#[from] SenseElementBuilderError),
    #[error(transparent)]
    CitElementBuilderError(#[from] CitElementBuilderError),
    #[error(transparent)]
    QuoteElementBuilderError(#[from] QuoteElementBuilderError),
    #[error(transparent)]
    GramGrpElementBuilderError(#[from] GramGrpElementBuilderError),
    #[error(transparent)]
    GenElementBuilderError(#[from] GenElementBuilderError),
    #[error(transparent)]
    UsgElementBuilderError(#[from] UsgElementBuilderError),
    #[error(transparent)]
    XrElementBuilderError(#[from] XrElementBuilderError),
    #[error(transparent)]
    NumberElementBuilderError(#[from] NumberElementBuilderError),
    #[error(transparent)]
    PosElementBuilderError(#[from] PosElementBuilderError),
    #[error(transparent)]
    SubcElementBuilderError(#[from] SubcElementBuilderError),
    #[error(transparent)]
    CollocElementBuilderError(#[from] CollocElementBuilderError),
    #[error(transparent)]
    TnsElementBuilderError(#[from] TnsElementBuilderError),
    #[error(transparent)]
    MoodElementBuilderError(#[from] MoodElementBuilderError),
}