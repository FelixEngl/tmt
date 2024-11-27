use xml2code::{generate_code_from_xml};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    const SKIP_HASH_TEST: bool = false;

    let result = vec! [
        std::thread::spawn(
            || {
                generate_code_from_xml!(
                    output: "src/dictionary/loader/helper/gen_freedict_tei_reader.rs",
                    panic_on_fail: false,
                    skip_hash_test: SKIP_HASH_TEST,
                    fail_if_analysis_fails: false,
                    analyze: r#"dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei"#,
                    analyze: r#"dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei"#,
                    set_type: "a_type" to RecognizedContentType::Enum,
                )
            }
        ),
        std::thread::spawn(
            || {
                generate_code_from_xml!(
                output: "src/dictionary/loader/helper/gen_iate_tbx_reader.rs",
                panic_on_fail: false,
                skip_hash_test: SKIP_HASH_TEST,
                fail_if_analysis_fails: true,
                analyze: r#"dictionaries/IATE/IATE_export.tbx"#,
                set_type: "a_type" to RecognizedContentType::Enum,
                set_type: "a_lang" to RecognizedContentType::Enum,
                set_type: "e_termNote" to RecognizedContentType::Enum,
            )
            }
        ),
        std::thread::spawn(
            || {
                generate_code_from_xml!(
                output: "src/dictionary/loader/helper/gen_ms_terms_reader.rs",
                panic_on_fail: false,
                skip_hash_test: SKIP_HASH_TEST,
                fail_if_analysis_fails: true,
                analyze: r#"dictionaries/Microsoft TermCollection/MicrosoftTermCollectio_british_englisch.tbx"#,
                analyze: r#"dictionaries/Microsoft TermCollection/MicrosoftTermCollection_german.tbx"#,
                set_type: "a_type" to RecognizedContentType::Enum,
                set_type: "a_lang" to RecognizedContentType::Enum,
                set_type: "e_termNote" to RecognizedContentType::Enum,
            )
            }
        )
    ];

    let result = result.into_iter().map(|value| value.join()).collect::<Vec<_>>();

    for value in result {
        value.expect("Thread had panic?").expect("The process failed.")
    }
}