macro_rules! take_bracket {
    ($open: literal, $close: literal) => {
        nom::sequence::delimited(
            nom::character::complete::char($open),
            nom::bytes::complete::take_until1(const_str::to_str!($close)),
            nom::character::complete::char($close)
        )
    };
}

pub(crate) use take_bracket;