use xml2code::generate_code_from_xml;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let result1 = std::thread::spawn(
        || {
            generate_code_from_xml!(
                output: "src/topicmodel/dictionary/loader/helper/gen_iate_tbx_reader.rs",
                panic_on_fail: false,
                fail_if_analysis_fails: true,
                analyze: r#"dictionaries/IATE/IATE_export.tbx"#,
            )
        }
    );

    let result2 = std::thread::spawn(
        || {
            generate_code_from_xml!(
                output: "src/topicmodel/dictionary/loader/helper/gen_freedict_tei_reader.rs",
                panic_on_fail: false,
                fail_if_analysis_fails: false,
                analyze: r#"dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei"#,
                analyze: r#"dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei"#,
            )
        }
    );

    let result3 = std::thread::spawn(
        || {
            generate_code_from_xml!(
                output: "src/topicmodel/dictionary/loader/helper/gen_ms_terms_reader.rs",
                panic_on_fail: false,
                fail_if_analysis_fails: true,
                analyze: r#"dictionaries/Microsoft TermCollection/MicrosoftTermCollectio_british_englisch.tbx"#,
                analyze: r#"dictionaries/Microsoft TermCollection/MicrosoftTermCollection_german.tbx"#,
            )
        }
    );

    let result = vec![
        result1.join(),
        result2.join(),
        result3.join()
    ];

    for value in result {
        value.expect("Thread had panic?").expect("The process failed.")
    }
}