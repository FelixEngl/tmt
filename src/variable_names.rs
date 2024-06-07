macro_rules! declare_variable_names_internal {
    ($variable_name: ident: $name: literal) => {
        pub const $variable_name: &str = $name;
    };

    (doc = $doc: literal $variable_name: ident: $name: literal) => {
        #[doc = $doc]
        pub const $variable_name: &str = $name;
    };

    ($variable_name: ident: $name: literal, $($tt:tt)+) => {
        pub const $variable_name: &str = $name;
        declare_variable_names_internal!($($tt)+);
    };

    (doc = $doc: literal $variable_name: ident: $name: literal, $($tt:tt)+) => {
        #[doc = $doc]
        pub const $variable_name: &str = $name;
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

macro_rules! declare_variable_names {
    ($variable_name: ident: $name: literal, $($tt:tt)*) => {
        declare_variable_names_internal!($variable_name: $name, $($tt)+);

        pub(crate) fn reserved_variable_name<'a, 'b, E: $crate::voting::parser::logic::ErrorType<$crate::voting::parser::input::ParserInput<'a, 'b>>>(input: $crate::voting::parser::input::ParserInput<'a, 'b>) -> nom::IResult<$crate::voting::parser::input::ParserInput<'a, 'b>, $crate::voting::parser::input::ParserInput<'a, 'b>,  E> {
            nom::sequence::preceded(
                nom::character::complete::multispace0,
                declare_alts!($variable_name: $name, $($tt)+)
            )(input)
        }
    };

    (doc = $doc: literal $variable_name: ident: $name: literal, $($tt:tt)*) => {
        declare_variable_names_internal!(doc = $doc $variable_name: $name, $($tt)+);

        pub(crate) fn reserved_variable_name<'a, 'b, E: $crate::voting::parser::logic::ErrorType<$crate::voting::parser::input::ParserInput<'a, 'b>>>(input: $crate::voting::parser::input::ParserInput<'a, 'b>) -> nom::IResult<$crate::voting::parser::input::ParserInput<'a, 'b>, $crate::voting::parser::input::ParserInput<'a, 'b>,  E> {
            nom::sequence::preceded(
                nom::character::complete::multispace0,
                declare_alts!($variable_name: $name, $($tt)+)
            )(input)
        }
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
    TOPIC_ID: "topic_id"
}
