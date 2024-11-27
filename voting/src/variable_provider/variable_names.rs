//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

macro_rules! declare_variable_names_internal {
    ($variable_name: ident: $name: literal) => {
        pub const $variable_name: &str = $name;
        #[cfg(feature = "gen_python_api")]
        pyo3_stub_gen::module_variable!("ldatranslate.variable_names", stringify!($variable_name), &'static str);
    };

    (doc = $doc: literal $variable_name: ident: $name: literal) => {
        #[doc = $doc]
        pub const $variable_name: &str = $name;
        #[cfg(feature = "gen_python_api")]
        pyo3_stub_gen::module_variable!("ldatranslate.variable_names", stringify!($variable_name), &'static str);
    };

    ($variable_name: ident: $name: literal, $($tt:tt)+) => {
        pub const $variable_name: &str = $name;
        #[cfg(feature = "gen_python_api")]
        pyo3_stub_gen::module_variable!("ldatranslate.variable_names", stringify!($variable_name), &'static str);
        declare_variable_names_internal!($($tt)+);
    };

    (doc = $doc: literal $variable_name: ident: $name: literal, $($tt:tt)+) => {
        #[doc = $doc]
        pub const $variable_name: &str = $name;
        #[cfg(feature = "gen_python_api")]
        pyo3_stub_gen::module_variable!("ldatranslate.variable_names", stringify!($variable_name), &'static str);
        declare_variable_names_internal!($($tt)+);
    };
}

macro_rules! declare_alts {
    ($variable_name: ident: $name: literal) => {
        nom::bytes::complete::tag($variable_name)
    };

    (doc = $doc: literal $variable_name: ident: $name: literal) => {
        declare_alts!($variable_name: $name)
    };

    ($variable_name: ident: $name: literal, $($tt:tt)+) => {
        nom::branch::alt((
            declare_alts!($variable_name: $name),
            declare_alts!($($tt)+)
        ))
    };

    (doc = $doc: literal $variable_name: ident: $name: literal, $($tt:tt)+) => {
        declare_alts!($variable_name: $name, $($tt)+)
    };

}

macro_rules! declare_py_module {
    ($module: ident, $variable_name: ident: $name: literal) => {
        $module.add(stringify!($variable_name), $name)?;
    };

    ($module: ident, doc = $doc: literal $variable_name: ident: $name: literal) => {
        $module.add(stringify!($variable_name), $name)?;
    };

    ($module: ident, $variable_name: ident: $name: literal, $($tt:tt)+) => {
        declare_py_module!($module, $variable_name: $name);
        declare_py_module!($module, $($tt)+);
    };

    ($module: ident, doc = $doc: literal $variable_name: ident: $name: literal, $($tt:tt)+) => {
        declare_py_module!($module, $variable_name: $name, $($tt)+);
    };
}




macro_rules! declare_variable_names {
    ($variable_name: ident: $name: literal, $($tt:tt)*) => {
        declare_variable_names_internal!($variable_name: $name, $($tt)+);

        pub(crate) fn reserved_variable_name<'a, 'b, E: $crate::parser::logic::ErrorType<$crate::parser::input::ParserInput<'a, 'b>>>(input: $crate::parser::input::ParserInput<'a, 'b>) -> nom::IResult<$crate::parser::input::ParserInput<'a, 'b>, $crate::parser::input::ParserInput<'a, 'b>,  E> {
            nom::sequence::preceded(
                nom::character::complete::multispace0,
                declare_alts!($variable_name: $name, $($tt)+)
            )(input)
        }

        // pub(crate) fn register_py_variable_names_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
        //     let submodule = PyModule::new_bound(m.py(), "variable_names")?;
        //     declare_py_module!(submodule, $variable_name: $name, $($tt)+);
        //     m.add_submodule(&submodule)?;
        //     Ok(())
        // }

        const _: () = {
            use ldatranslate_toolkit;
            ldatranslate_toolkit::register_python! {
                custom(m) {
                    let submodule = PyModule::new_bound(m.py(), "variable_names")?;
                    declare_py_module!(submodule, $variable_name: $name, $($tt)+);
                    m.add_submodule(&submodule)?;
                }
            }
        };
    };

    (doc = $doc: literal $variable_name: ident: $name: literal, $($tt:tt)*) => {
        declare_variable_names_internal!(doc = $doc $variable_name: $name, $($tt)+);

        pub(crate) fn reserved_variable_name<'a, 'b, E: $crate::parser::logic::ErrorType<$crate::parser::input::ParserInput<'a, 'b>>>(input: $crate::parser::input::ParserInput<'a, 'b>) -> nom::IResult<$crate::parser::input::ParserInput<'a, 'b>, $crate::parser::input::ParserInput<'a, 'b>,  E> {
            nom::sequence::preceded(
                nom::character::complete::multispace0,
                declare_alts!($variable_name: $name, $($tt)+)
            )(input)
        }

        // pub(crate) fn register_py_variable_names_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
        //     let submodule = PyModule::new_bound(m.py(), "variable_names")?;
        //     declare_py_module!(submodule, doc = $doc $variable_name: $name, $($tt)+);
        //     m.add_submodule(&submodule)?;
        //     Ok(())
        // }
        const _:() = {
            use ldatranslate_toolkit;
            ldatranslate_toolkit::register_python! {
                custom(m) {
                    let submodule = PyModule::new_bound(m.py(), "variable_names")?;
                    declare_py_module!(submodule, doc = $doc $variable_name: $name, $($tt)+);
                    m.add_submodule(&submodule)?;
                }
            }
        };
    };
}


declare_variable_names! {
    doc = "The epsilon of the calculation."
    EPSILON: "epsilon",
    doc = "The size of the vocabulary in language a."
    VOCABULARY_SIZE_A: "n_voc",
    doc = "The size of the vocabulary in language b."
    VOCABULARY_SIZE_B: "n_voc_target",
    doc = "The max probability of the topic."
    TOPIC_MAX_PROBABILITY: "topic_max",
    doc = "The min probability of the topic."
    TOPIC_MIN_PROBABILITY: "topic_min",
    doc = "The avg probability of the topic."
    TOPIC_AVG_PROBABILITY: "topic_avg",
    doc = "The sum of all probabilities of the topic."
    TOPIC_SUM_PROBABILITY: "topic_sum",
    doc = "The number of available voters"
    COUNT_OF_VOTERS: "ct_voters",
    doc = "The number of used voters."
    NUMBER_OF_VOTERS: "n_voters",
    doc = "True if the word in language A has translations to language B."
    HAS_TRANSLATION: "has_translation",
    doc = "True if this is the original word in language A"
    IS_ORIGIN_WORD: "is_origin_word",
    doc = "The original score of the candidate."
    SCORE_CANDIDATE: "score_candidate",
    doc = "The reciprocal rank of the word."
    RECIPROCAL_RANK: "rr",
    doc = "The real reciprocal rank of the word."
    REAL_RECIPROCAL_RANK: "rr_real",
    doc = "The rank of the word."
    RANK: "rank",
    doc = "The importance rank of the word."
    IMPORTANCE: "importance",
    doc = "The score of the word in the topic model."
    SCORE: "score",
    doc = "The word id of a voter."
    VOTER_ID: "voter_id",
    doc = "The word id of a candidate."
    CANDIDATE_ID: "candidate_id",
    doc = "The topic id."
    TOPIC_ID: "topic_id",
    doc = "The score of the domain for this voter in this topic."
    SCORE_DOMAIN: "domain_score",
    doc = "A boost variable that is applied when set. Can be a value or an other field name."
    BOOST: "boost"
}

