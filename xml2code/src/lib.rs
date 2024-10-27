use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use base64::Engine;
use itertools::Itertools;
use twox_hash::xxhash3_64::SecretBuffer;

mod hashref;
mod converter;

pub use converter::{RecognizedContentType};
use crate::converter::{XML2CodeConverter};
use crate::error::{Error};

/// Allows to import all types necessary for error handling


#[doc(hidden)]
pub mod macro_impl {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use derive_builder::Builder;
    use crate::RecognizedContentType;

    #[derive(Builder, Clone)]
    pub struct GenerateCodeCommand {
        #[builder(setter(custom))]
        pub output: PathBuf,
        #[builder(setter(custom))]
        pub inputs: Vec<PathBuf>,
        #[builder(setter(custom), default)]
        pub mappings: Option<HashMap<String, RecognizedContentType>>,
        #[builder(default)]
        pub fail_if_analysis_fails: bool,
        #[builder(default)]
        pub ignore_hash_test: bool
    }

    impl GenerateCodeCommandBuilder {
        pub fn output(&mut self, p: impl AsRef<Path>) -> Result<(), GenerateCodeCommandBuilderError> {
            if let Some(old) = self.output.replace(p.as_ref().to_path_buf()) {
                return Err(GenerateCodeCommandBuilderError::ValidationError(format!("The output was already set with {}", old.to_string_lossy())))
            }
            Ok(())
        }

        pub fn input(&mut self, p: impl AsRef<Path>) {
            self.inputs.get_or_insert_with(Default::default).push(p.as_ref().to_path_buf());
        }

        pub fn mapping_entry(&mut self, k: impl Into<String>, v: RecognizedContentType) {
            let map = self.mappings.get_or_insert_with(|| Some(HashMap::new())).as_mut().unwrap();
            map.insert(k.into(), v);
        }
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! __private_generate_code {
        (
            output: $output: expr,
            panic_on_fail: false,
            $($tt:tt)+
        ) => {
            {
                fn exec() -> Result<(), $crate::error::Error> {
                    use $crate::macro_impl::GenerateCodeCommandBuilder;
                    use $crate::RecognizedContentType;
                    use $crate::macro_impl::GenerateCodeCommand;
                    use $crate::generate_code;
                    let mut builder = GenerateCodeCommandBuilder::default();
                    builder.output($output).unwrap();
                    $crate::__private_generate_code!(__private(builder) $($tt)+);

                    let command: GenerateCodeCommand = builder.build()?;
                    generate_code(
                        command.output,
                        command.inputs,
                        command.mappings.as_ref(),
                        command.fail_if_analysis_fails,
                        command.ignore_hash_test
                    )
                }
                exec()
            }
        };
        (
            output: $output: expr,
            panic_on_fail: true,
            $($tt:tt)+
        ) => {
            {
                let result = $crate::__private_generate_code!(
                    output: $output,
                    panic_on_fail: false,
                    $($tt)+
                );
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        panic!("Failed with: {}", err);
                    }
                }
            }
        };
        (__private($builder: ident) fail_if_analysis_fails: $fail_if_analysis_fails: expr, $($tt:tt)*) => {
            $builder.fail_if_analysis_fails($fail_if_analysis_fails);
            $crate::__private_generate_code!(__private($builder) $($tt)*);
        };
        (__private($builder: ident) skip_hash_test: $ignore_hash_test: expr, $($tt:tt)*) => {
            $builder.ignore_hash_test($ignore_hash_test);
            $crate::__private_generate_code!(__private($builder) $($tt)*);
        };
        (__private($builder: ident) analyze: $path_like: expr, $($tt:tt)*) => {
            $builder.input($path_like);
            $crate::__private_generate_code!(__private($builder) $($tt)*);
        };
        (__private($builder: ident) set_type: $k: literal to $v: expr, $($tt:tt)*) => {
            $builder.mapping_entry($k, $v);
            $crate::__private_generate_code!(__private($builder) $($tt)*);
        };
        (__private($builder: ident)) => {};
    }
}

pub mod error {
    pub use crate::hashref::HashRef;
    pub use super::converter::{GenericXMLParserError, XML2CodeConverterError, CodeAttribute, CodeElement};
    pub use quick_xml::{Error as XmlError, events::attributes::AttrError};
    pub use std::str::{Utf8Error, ParseBoolError};
    pub use std::num::{ParseIntError, ParseFloatError};
    use std::path::PathBuf;
    pub use strum::ParseError;
    use thiserror::Error;
    pub use super::macro_impl::GenerateCodeCommandBuilderError;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error(transparent)]
        OutputFile(std::io::Error),
        #[error("Failed {0:?} with {1}")]
        InputFile(PathBuf, std::io::Error),
        #[error(transparent)]
        Analysis(#[from] XML2CodeConverterError),
        #[error(transparent)]
        CodeGen(std::io::Error),
        #[error(transparent)]
        HashFile(std::io::Error),
        #[error(transparent)]
        CommandBuilder(#[from] GenerateCodeCommandBuilderError),
        #[error("You need at least one input file!")]
        NoInputFile
    }
}

/// A macro to generate code from xml-files.
/// This macro takes the following arguments:
/// output: "<path to file to be generated>", (Has to be the first argument.)
/// panic_on_fail: true | false ,(default: true, if false the macro returns a result. Has to be placed below the output: argument.)
/// fail_if_analysis_fails: true | false, (default: false)
/// skip_hash_test: true | false, (default: false)
/// analyze: <path to xml file for analysis>, (Needs to be used at least one time. Can be used multiple times.)
/// set_type: "element or attribute name" to [RecognizedContentType], (Optional. Can be used multiple times. Allows to set a specific type for a specific element/attribute.)
#[macro_export]
macro_rules! generate_code_from_xml {
    (
        output: $output: expr,
        panic_on_fail: false,
        $($tt:tt)+
    ) => {
        $crate::__private_generate_code!(
           output: $output,
           panic_on_fail: false,
           $($tt)+
        )
    };
    (
        output: $output: expr,
        panic_on_fail: true,
        $($tt:tt)+
    ) => {
        $crate::__private_generate_code!(
           output: $output,
           panic_on_fail: true,
           $($tt)+
        )
    };
    (
        output: $output: expr,
        $($tt:tt)+
    ) => {
        $crate::generate_code_from_xml!(
            output: $output,
            panic_on_fail: true,
            $($tt)+
        )
    };
}


const PREFIX_LEN: usize = 17;
const HASH_LEN: usize = PREFIX_LEN + match base64::encoded_len(16, true) {
    None => {panic!("Failed sise calc encoding")}
    Some(value) => {value}
};
const PREFIX: &[u8; PREFIX_LEN] = b"//hash_signature:";

/// Panics with assertion error if  the inputs are not sorted by to_string_lossy
fn create_hash<P>(inputs: &[P]) -> Result<[u8; HASH_LEN], error::Error>
where
    P: AsRef<Path>,
{
    assert!(inputs.is_sorted_by_key(|value| value.as_ref().to_string_lossy()));
    let mut len_hasher = twox_hash::xxhash3_64::Hasher::new();
    let mut content_hasher = twox_hash::xxhash3_64::RawHasher::new(SecretBuffer::allocate_default());
    for path in inputs.iter() {
        match File::options().read(true).open(path.as_ref()) {
            Ok(file) => {
                let meta = file.metadata().map_err(|err| Error::InputFile(path.as_ref().to_path_buf(), err))?;
                meta.len().hash(&mut len_hasher);
                let mut reader = BufReader::with_capacity(1024*128, file);
                loop {
                    let consumed = {
                        match reader.fill_buf() {
                            Ok(value) => {
                                if value.is_empty() {
                                    break
                                }
                                content_hasher.write(value);
                                value.len()
                            }
                            Err(err) => {
                                return Err(Error::InputFile(path.as_ref().to_path_buf(), err))
                            }
                        }
                    };
                    reader.consume(consumed);
                }
            }
            Err(err) => {
                log::error!("Failed opening file {}: {err}", path.as_ref().to_string_lossy());
                return Err(Error::InputFile(path.as_ref().to_path_buf(), err))
            }
        }
    }

    let mut hash = [0u8; HASH_LEN];
    (&mut hash[..PREFIX.len()]).copy_from_slice(PREFIX);
    let mut data = [0u8; 16];
    (&mut data[..8]).copy_from_slice(&len_hasher.finish().to_be_bytes());
    (&mut data[8..]).copy_from_slice(&content_hasher.finish().to_be_bytes());
    base64::engine::general_purpose::STANDARD.encode_slice(data, &mut hash[PREFIX.len()..]).expect("This never fails!");
    Ok(hash)
}


fn read_hash(path: impl AsRef<Path>) -> Result<Option<[u8; HASH_LEN]>, error::Error> {
    if path.as_ref().exists() {
        let mut buf = [0u8; HASH_LEN];
        match File::options()
            .read(true).open(path.as_ref()).map_err(Error::HashFile)?
            .read_exact(buf.as_mut())
        {
            Ok(_) => {
                Ok(Some(buf))
            }
            Err(_) => {
                Ok(None)
            }
        }

    } else {
        Ok(None)
    }
}

pub fn generate_code<K, I, P>(
    output: impl AsRef<Path>,
    inputs: I,
    mappings: Option<&HashMap<K, RecognizedContentType>>,
    fail_if_analysis_fails: bool,
    ignore_hash_test: bool
) -> Result<(), error::Error>
where
    K: Borrow<str> + Eq + Hash,
    I: IntoIterator<Item=P>,
    P: AsRef<Path>,
{
    let mut inputs = inputs.into_iter().collect_vec();
    if inputs.is_empty() {
        return Err(Error::NoInputFile)
    }
    inputs.sort_by_key(|value| value.as_ref().to_string_lossy().into_owned());
    let hash: [u8; HASH_LEN] = create_hash(inputs.as_slice())?;

    if !ignore_hash_test {
        let old_hash: Option<[u8; HASH_LEN]> = read_hash(output.as_ref())?;
        if let Some(old_hash) = old_hash {
            if old_hash.eq(&hash) {
                log::info!("The hash signatures of the existing generated file and the calculated hash value are the same.");
                return Ok(())
            }
        }
    }

    let mut ct_success = 0;
    let mut result = XML2CodeConverter::default();
    for path in inputs.iter() {
        match File::options().read(true).open(path.as_ref()) {
            Ok(file) => {
                let buf = BufReader::with_capacity(
                    1024*128,
                    file
                );
                let mut reader = quick_xml::reader::Reader::from_reader(buf);
                match result.analyze(&mut reader) {
                    Ok(_) => {
                        ct_success += 1;
                    }
                    Err(err) => {
                        log::error!("Failed analyzing {}: {err}", path.as_ref().to_string_lossy());
                        if fail_if_analysis_fails {
                            return Err(err.into())
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("Failed opening file {}: {err}", path.as_ref().to_string_lossy());
                return Err(Error::InputFile(path.as_ref().to_path_buf(), err))
            }
        }
    }
    if ct_success < inputs.len() {
        log::warn!("Not all files where processed!");
    }

    let mut writer = Vec::new();
    let written = writer.write(hash.as_slice()).map_err(Error::CodeGen)?;
    if written != hash.len() {
        log::warn!("Failed to write the hash to the file!");
    }
    write!(&mut writer, "\n\n").map_err(Error::CodeGen)?;

    if let Some(mapping) = mappings {
        match result.generate_code(&mut writer, mapping) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed generating the code: {err}");
                return Err(Error::CodeGen(err))
            }
        }
    } else {
        match result.generate_code(&mut writer, &HashMap::<&str, _>::with_capacity(0)) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed generating the code: {err}");
                return Err(Error::CodeGen(err))
            }
        }
    }

    let output_file = match File::options().write(true).truncate(true).create(true).open(&output) {
        Ok(f) => {
            f
        }
        Err(err) => {
            log::error!("Was not able to create the output at {}: {err}", output.as_ref().to_string_lossy());
            return Err(Error::OutputFile(err));
        }
    };
    let mut file_writer = BufWriter::with_capacity(1024*128, output_file);
    file_writer.write_all(writer.as_slice()).map_err(|v| Error::OutputFile(v))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use crate::create_hash;

    #[test]
    pub fn macro_works_as_expected(){
        generate_code_from_xml!(
            output: "test.rs",
            fail_if_analysis_fails: false,
            skip_hash_test: true,
            analyze: r#"../dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei"#,
            analyze: r#"../dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei"#,
        )
    }

    #[test]
    pub fn test_hash() {
        const TARGETS: &[&str] = &[
            r#"../dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei"#,
            r#"../dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei"#,
        ];
        let mut inputs = Vec::from(TARGETS);
        inputs.sort_by_key(|value| AsRef::<Path>::as_ref(*value).to_string_lossy().into_owned());
        let hash = create_hash(&inputs).unwrap();
        println!("{}", std::str::from_utf8(&hash).unwrap());
    }
}