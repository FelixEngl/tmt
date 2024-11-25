use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};
use camino::{Utf8Path, Utf8PathBuf};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, DictionaryWithMeta, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::direction::{Language, A, B};
use crate::topicmodel::dictionary::metadata::{MetadataManagerGen};
use crate::topicmodel::vocabulary::{AlphabeticalVocabulary, AnonymousVocabulary, BasicVocabulary};
use build_html::{Container, Html, HtmlContainer, HtmlPage, Table, TableCell, TableRow};
use build_html::ContainerType::*;
use itertools::Itertools;
use rayon::prelude::*;
use crate::topicmodel::dictionary::metadata::ex::{LoadedMetadataEx, MetadataManagerEx, SolvedMetadataField};


macro_rules! div {
    // (class = $lit: literal $b: expr $(;+ $other: expr)* $(;)?) => {
    //     Container::default().with_attributes([("class", $lit)])
    //         .with_container($b)
    //         $(.with_container($other))*
    // };
    (class = $lit: literal { + $b: expr $(;+ $other: expr)* $(;)? }) => {
        Container::default().with_attributes([("class", $lit)])
            .with_container($b)
            $(.with_container($other))*
    };
    (class = $lit: literal) => {
        Container::default().with_attributes([("class", $lit)])
    };
    (attrs = [$attrs: expr] { + $b: expr $(;+ $other: expr)* $(;)? }) => {
        Container::default().with_attributes($attrs)
        .with_container($b)
        $(.with_container($other))*
    };
    (+ $b: expr $(;+ $other: expr)* $(;)?) => {
        Container::default()
        .with_container($b)
        $(.with_container($other))*
    };
    () => {
        Container::default()
    };
}

use div;
use crate::toolkit::crc32;

impl<T, V> DictionaryWithMeta<T, V, MetadataManagerEx>
where
    T: Ord + Display + Clone + Send + Sync,
    V: BasicVocabulary<T> + AnonymousVocabulary + Send + Sync,
{

    fn generate_directed<L: Language>(&self, main: impl AsRef<Utf8Path>, dir: impl AsRef<Utf8Path>, use_crc32: bool) -> Result<Vec<(T, T, Utf8PathBuf)>, std::io::Error> {

        let main = main.as_ref();
        std::fs::create_dir_all(dir.as_ref())?;

        let dir = dir.as_ref().canonicalize_utf8()?;

        let sorted_ids: Vec<usize>;
        let mapping: &Vec<Vec<usize>>;
        let voc_origin: &V;
        let voc_target: &V;

        if L::LANG.is_a() {
            sorted_ids = self.voc_a().ids_in_alphabetical_order();
            mapping = self.map_a_to_b();
            voc_origin = self.voc_a();
            voc_target = self.voc_b();
        } else {
            sorted_ids = self.voc_b().ids_in_alphabetical_order();
            mapping = self.map_b_to_a();
            voc_origin = self.voc_b();
            voc_target = self.voc_a();
        }


        let pages = sorted_ids.into_par_iter().chunks(200).map(|chunk| {
            let start = *chunk.first().unwrap();
            let end = *chunk.last().unwrap();
            let file_path = dir.join(format!("{}_{}_{}.html", L::LANG, start, end));
            let start = voc_origin.get_value_by_id(start).unwrap().clone();
            let end = voc_origin.get_value_by_id(end).unwrap().clone();
            let is_same = if use_crc32 && file_path.exists() {
                File::options().read(true).open(&file_path).map(BufReader::new).and_then(|value| {
                    crc32(value).map(Some)
                })
            } else {
                Ok(None)
            };
            (file_path, start, end, is_same, chunk)
        }).map(|(file_path, start, end, crc, chunk)| {
            crc.and_then(|crc| {
                let html = HtmlPage::new()
                    .with_title(
                        format!(
                            "{} to {}",
                            start,
                            end
                        )
                    )
                    .with_head_link_attr(
                        "https://cdn.jsdelivr.net/npm/bootstrap@3.4.1/dist/css/bootstrap.min.css",
                        "stylesheet",
                        [
                            ("integrity", "sha384-HSMxcRTRxnN+Bdg0JdbxYKrThecOKuH5zCYotlSAcp1+c8xmyTe9GYg1l9a69psu"),
                            ("crossorigin", "anonymous"),
                        ]
                    )
                    .with_container(
                        Container::new(Main)
                            .with_container(
                                div! {
                                class = "container-fluid"
                                {
                                    + div!(class="row").with_link(pathdiff::diff_utf8_paths(&main, &dir).expect("This should never fail"), "Main");
                                    + div! {
                                        class="row" {
                                            + div!(class="col-xs-3").with_header(3, voc_origin.language().map(|value| value.to_string()).unwrap_or_else(|| L::LANG.to_string()));
                                            + div!(class="col-xs-3").with_header(3, voc_target.language().map(|value| value.to_string()).unwrap_or_else(|| L::OPPOSITE::LANG.to_string()));
                                            + div!(class="col-xs-6").with_header(3, "Metadata");
                                        }
                                    };
                                    + {
                                        let mut table = div!();
                                        for id_a in chunk.iter().copied() {
                                            let mut translations = unsafe{
                                                mapping.get_unchecked(id_a).into_iter().map(|value| format!("{} ({})", voc_target.get_value_unchecked(*value), value))
                                            }.collect_vec();
                                            translations.sort();
                                            table.add_container(
                                                div! {
                                                    class="row" {
                                                        + div!(class="col-xs-3").with_raw(unsafe{format!("{} ({})", voc_origin.get_value_unchecked(id_a), id_a)});
                                                        + div!{
                                                            class="col-xs-3" {
                                                                + {
                                                                    let mut cont = Container::new(UnorderedList).with_attributes(
                                                                        [("class", "list-unstyled")]
                                                                    );
                                                                    for t in translations {
                                                                        cont.add_paragraph(t);
                                                                    }
                                                                    cont
                                                                }
                                                            }
                                                        };
                                                        + div! {
                                                            class="col-xs-6" {
                                                                + {
                                                                    let v: &dyn AnonymousVocabulary = voc_origin;
                                                                    if let Some(cont) = self.metadata().get_meta_ref::<L>(v, id_a) {
                                                                        Container::default().with_table(
                                                                            {
                                                                                let mut subtab = Table::new()
                                                                                    .with_attributes([("class", "table table-bordered")])
                                                                                    .with_header_row(["MetaField", "Origin", "Data"]);
                                                                                let mut loaded = LoadedMetadataEx::from(cont).as_dict().into_iter().collect_vec();
                                                                                loaded.sort_by_key(|value| value.0);
                                                                                for (field, SolvedMetadataField(general, dict)) in loaded {
                                                                                    if let Some(general) = general  {
                                                                                        if !general.is_empty() {
                                                                                            let mut entry = Vec::from_iter(general);
                                                                                            entry.sort();
                                                                                            subtab.add_custom_body_row(
                                                                                                TableRow::new()
                                                                                                    .with_cell(TableCell::default().with_raw(field))
                                                                                                    .with_cell(TableCell::default().with_raw("General"))
                                                                                                    .with_cell(TableCell::default().with_raw(format!("\"{}\"", entry.into_iter().join("\", \""))))
                                                                                            );
                                                                                        }
                                                                                    }
                                                                                    if let Some(dict) = dict {
                                                                                        let mut dict = Vec::from_iter(dict);
                                                                                        dict.sort_by_key(|v| v.0.clone());
                                                                                        for (dict, entries) in dict {
                                                                                            if entries.is_empty() {
                                                                                                continue
                                                                                            }
                                                                                            let mut entry = Vec::from_iter(entries);
                                                                                            entry.sort();
                                                                                            subtab.add_custom_body_row(
                                                                                                TableRow::new()
                                                                                                    .with_cell(TableCell::default().with_raw(field))
                                                                                                    .with_cell(TableCell::default().with_raw(dict))
                                                                                                    .with_cell(TableCell::default().with_raw(format!("\"{}\"", entry.into_iter().join("\", \""))))
                                                                                            );
                                                                                        }
                                                                                    }
                                                                                }
                                                                                subtab
                                                                            }
                                                                        )
                                                                    } else {
                                                                        Container::default().with_paragraph("No metadata!")
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            )
                                        }
                                        table
                                    }
                                }
                            }
                            )
                    )
                    .to_html_string();

                let is_same = if let Some(crc) = crc {
                    let checksum = crc32(Cursor::new(html.as_bytes()))?;
                    checksum == crc
                } else {
                    false
                };

                if is_same {
                    log::info!("Keep: {} (crc32 {:#08x})", file_path, crc.unwrap());
                    Ok((start, end, file_path))
                } else {
                    log::info!("Write: {}", file_path);
                    File::options().create(true).truncate(true).write(true).open(&file_path)
                        .and_then(|file| BufWriter::new(file).write_all(html.as_bytes()))
                        .map(|_|(start, end, file_path))
                }
            })
        }).collect::<Result<Vec<_>, _>>();

        pages
    }

    fn generate_statistics(&self, _: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf, std::io::Error> {
        HtmlPage::new()
            .with_title("Metadata")
            .with_head_link_attr(
                "https://cdn.jsdelivr.net/npm/bootstrap@3.4.1/dist/css/bootstrap.min.css",
                "stylesheet",
                [
                    ("integrity", "sha384-HSMxcRTRxnN+Bdg0JdbxYKrThecOKuH5zCYotlSAcp1+c8xmyTe9GYg1l9a69psu"),
                    ("crossorigin", "anonymous"),
                ]
            )
            .with_container(div! {
                class="container-fluid" {
                    + div! {
                        class="row" {
                            + div! {
                                class="mx-auto" {
                                    + div!{
                                        + {
                                            let x = self.len();
                                            div! {
                                                class="row" {
                                                    + div!(class="col-xs-3").with_paragraph(format!("Vocabulary A: {}", x.voc_a));
                                                    + div!(class="col-xs-3").with_paragraph(format!("Vocabulary B: {}", x.voc_b));
                                                    + div!(class="col-xs-3").with_paragraph(format!("Map A to B: {}", x.map_a_to_b));
                                                    + div!(class="col-xs-3").with_paragraph(format!("Map B to A: {}", x.map_b_to_a));
                                                }
                                            }
                                        };
                                    }
                                }
                            }
                        }
                    };
                    + div! {
                        class="row" {
                            + div! {

                            }
                        }
                    };
                }
            })  ;


        todo!()
    }

    pub fn generate_html(&self, dir: impl AsRef<Utf8Path>, use_crc32: bool) -> Result<(), std::io::Error> {
        let d = dir.as_ref();
        std::fs::create_dir_all(d)?;
        let main = d.canonicalize_utf8()?.join("index.html");
        let a = self.generate_directed::<A>(&main, d.join("A"), use_crc32)?;
        let b = self.generate_directed::<B>(&main, d.join("B"), use_crc32)?;
        log::info!("Generate index.html");
        let d = d.canonicalize_utf8()?;
        let html = HtmlPage::new()
            .with_title("Overview of dict.")
            .with_head_link_attr(
                "https://cdn.jsdelivr.net/npm/bootstrap@3.4.1/dist/css/bootstrap.min.css",
                "stylesheet",
                [
                    ("integrity", "sha384-HSMxcRTRxnN+Bdg0JdbxYKrThecOKuH5zCYotlSAcp1+c8xmyTe9GYg1l9a69psu"),
                    ("crossorigin", "anonymous"),
                ]
            )
            .with_container(
                div! {
                    class="container-fluid" {
                        + div! {
                            class = "row" {
                                + div!(class="col-xs-6")
                                    .with_header(
                                        2,
                                        format!(
                                            "{} to {} ({})",
                                            self.voc_a().language().map(|value| value.to_string()).unwrap_or_else(|| "A".to_string()),
                                            self.voc_b().language().map(|value| value.to_string()).unwrap_or_else(|| "B".to_string()),
                                            self.map_a_to_b().len()
                                        )
                                    )
                                    .with_container(
                                        {
                                            let mut data = Container::new(UnorderedList).with_attributes([("class", "list-unstyled")]);
                                            for (start, end, path) in a {
                                                 data.add_link(pathdiff::diff_utf8_paths(path, &d).unwrap(), format!("{} .. {}", start, end))
                                            }
                                            data
                                         }
                                    );
                                + div!(class="col-xs-6")
                                    .with_header(
                                        2,
                                        format!(
                                            "{} to {} ({})",
                                            self.voc_b().language().map(|value| value.to_string()).unwrap_or_else(|| "B".to_string()),
                                            self.voc_a().language().map(|value| value.to_string()).unwrap_or_else(|| "A".to_string()),
                                            self.map_b_to_a().len()
                                        )
                                    )
                                    .with_container({
                                        let mut data = Container::new(UnorderedList).with_attributes([("class", "list-unstyled")]);
                                        for (start, end, path) in b {
                                            data.add_link(pathdiff::diff_utf8_paths(path, &d).unwrap(), format!("{} .. {}", start, end))
                                        }
                                        data
                                    });
                            }
                        }
                    }
                }

            )
            .to_html_string();
        let mut outp = BufWriter::new(File::options().write(true).create(true).truncate(true).open(main)?);
        outp.write_all(html.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use crate::py::dictionary::{DefaultDict};
    use crate::topicmodel::dictionary::io::ReadableDictionary;

    #[test]
    fn can_generate_html(){
        let data = DefaultDict::from_path_with_extension("dictionary2.dat.zst").unwrap();
        data.generate_html(
            "E:/tmp/dict_view2",
            true
        ).unwrap()
    }
}
