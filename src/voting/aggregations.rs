use std::fmt::{Debug, Display, Formatter};
use std::iter::Sum;
use std::num::{NonZeroUsize};
use std::ops::Add;
use itertools::{Itertools};
use num::{Num};
use num::traits::{AsPrimitive, ConstZero};
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::toolkit::normal_number::IsNormalNumber;
use crate::toolkit::partial_ord_iterator::PartialOrderIterator;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Aggregation {
    typ: AggregationType,
    limit: Option<NonZeroUsize>,
}

impl Aggregation {
    pub const fn new_no_limit(
        typ: AggregationType,
    ) -> Self {
        Self::new(typ, None)
    }

    pub const fn new_with_limit(
        typ: AggregationType,
        limit: usize,
    ) -> Option<Self> {
        if let Some(value) = NonZeroUsize::new(limit) {
            Some(Self::new(typ, Some(value)))
        } else {
            None
        }
    }

    pub const unsafe fn new_with_limit_unchecked(
        typ: AggregationType,
        limit: usize,
    ) -> Self {
        Self::new(typ, Some(NonZeroUsize::new_unchecked(limit)))
    }

    pub const fn new(
        typ: AggregationType,
        limit: Option<NonZeroUsize>
    ) -> Self {
        Self {
            typ,
            limit,
        }
    }

    pub fn calculate_asc<T, I>(&self, iterator: I) -> Result<f64, AggregationError>
        where
            T: Num + PartialOrd + IsNormalNumber + ConstZero + AsPrimitive<f64> + Add + Sum,
            I: Iterator<Item=T>,
    {
        if let Some(limit) = self.limit {
            let mut vec = iterator
                .filter(|value| value.is_normal_number())
                .collect_vec();
            vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
            self.typ.calculate::<T, _>(vec.into_iter().take(limit.get()))
        } else {
            self.typ.calculate(iterator)
        }
    }


    pub fn calculate_desc<T, I>(&self, iterator: I) -> Result<f64, AggregationError>
        where
            T: Num + PartialOrd + IsNormalNumber + ConstZero + AsPrimitive<f64> + Add + Sum,
            I: Iterator<Item=T>,
    {
        if let Some(limit) = self.limit {
            let mut vec = iterator
                .filter(|value| value.is_normal_number())
                .collect_vec();
            vec.sort_by(|a, b| b.partial_cmp(a).unwrap());
            self.typ.calculate::<T, _>(vec.into_iter().take(limit.get()))
        } else {
            self.typ.calculate(iterator)
        }
    }
}



impl Display for Aggregation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(limit) = self.limit {
            write!(f, "{}({})", self.typ, limit)
        } else {
            write!(f, "{}", self.typ)
        }
    }
}


#[derive(Debug, Copy, Clone, Error, PartialEq)]
pub enum AggregationError {
    #[error("There is no value to be used!")]
    NoValues,
    #[error("There is no max value!")]
    NoMaxFound,
    #[error("There is no min value!")]
    NoMinFound,
}


#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
#[derive(AsRefStr, Display, EnumString)]
pub enum AggregationType {
    #[strum(serialize = "sumOf")]
    SumOf,
    #[strum(serialize = "maxOf")]
    MaxOf,
    #[strum(serialize = "minOf")]
    MinOf,
    #[strum(serialize = "avgOf")]
    AvgOf,
    #[strum(serialize = "gAvgOf")]
    GAvgOf,
}


impl AggregationType {
    pub fn calculate<T, I>(&self, iter: I) -> Result<f64, AggregationError>
        where
            T: Num + PartialOrd + IsNormalNumber + ConstZero + AsPrimitive<f64> + Add + Sum,
            I: Iterator<Item=T>,
    {

        let mut iter = match iter.at_most_one() {
            Ok(None) => {
                return Err(AggregationError::NoValues)
            }
            Ok(Some(value)) => {
                return Ok(value.as_())
            }
            Err(err) => {
                err
            }
        };


        fn calc_average<T, I>(iter: &mut I) -> f64
            where
                T: Num + PartialOrd + IsNormalNumber + ConstZero + AsPrimitive<f64> + Add,
                I: Iterator<Item=T>, {
            let mut value = T::ZERO;
            let mut ct = 0usize;
            while let Some(current) = iter.next() {
                value = value + current;
                ct += 1;
            }
            value.as_() / (ct as f64)
        }


        match self {
            AggregationType::SumOf => {
                Ok(iter.sum::<T>().as_())
            }
            AggregationType::MaxOf => {
                match iter.max_partial_filtered() {
                    Some(value) => {
                        Ok(value.as_())
                    }
                    None => {
                        Err(AggregationError::NoMaxFound)
                    }
                }
            }
            AggregationType::MinOf => {
                match iter.min_partial_filtered() {
                    Some(value) => {
                        Ok(value.as_())
                    }
                    None => {
                        Err(AggregationError::NoMinFound)
                    }
                }
            }
            AggregationType::AvgOf => {
                Ok(calc_average(&mut iter))
            }
            AggregationType::GAvgOf => {
                let mut iter = iter.map(|value| value.as_().ln());
                let avg = calc_average(&mut iter);
                Ok(avg.exp())
            }
        }
    }
}

pub mod parse {
    use std::num::{NonZeroUsize, ParseIntError};
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{alpha1, digit1, multispace0};
    use nom::combinator::{map, map_res, opt, value};
    use nom::error::context;
    use nom::IResult;
    use nom::sequence::{delimited, preceded, terminated, tuple};
    use thiserror::Error;
    use crate::toolkit::nom::ws;
    use crate::voting::aggregations::{Aggregation, AggregationType};
    use crate::voting::parse::{ErrorType};

    #[derive(Debug, Clone, Error)]
    pub enum AggregationParserError {
        #[error(transparent)]
        UnknownAggregation(#[from] strum::ParseError),
        #[error(transparent)]
        InvalidNumber(#[from] ParseIntError)
    }

    /// Parses a aggregation from a string.
    /// The syntax is `BulkOperationType`(limit)
    /// e.g. ``avgOf`` or ``sumOf(3)``
    /// Also supports legacy expressions like   ``sumOf limit(*)``
    pub fn parse_aggregation<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, Aggregation, E> {
        context(
            "aggregation",
            map(
                tuple((
                    map_res(
                        ws(alpha1),
                        |value|
                            AggregationType::try_from(value)
                                .map_err(AggregationParserError::UnknownAggregation)
                    ),
                    opt(
                        preceded(
                            multispace0,
                            preceded(
                                opt(terminated(tag("limit"), multispace0)),
                                delimited(
                                    tag("("),
                                    alt((
                                        map_res(digit1, |value: &str| match value.parse::<NonZeroUsize>() {
                                            Ok(value) => {Ok(Some(value))}
                                            Err(value) => {Err(AggregationParserError::InvalidNumber(value))}
                                        }),
                                        value(None, tag("*"))
                                    )),
                                    tag(")")
                                )
                            )
                        )
                    )
                )),
                |(typ, limit)| Aggregation::new(typ, limit.flatten())
            )
        )(input)
    }

    #[cfg(test)]
    mod test {
        use nom::error::VerboseError;
        use crate::voting::aggregations::{Aggregation, AggregationType};
        use crate::voting::aggregations::parse::parse_aggregation;

        #[test]
        fn can_parse_a_simple_expression(){
            assert_eq!(
                Aggregation::new_no_limit(AggregationType::SumOf),
                parse_aggregation::<VerboseError<_>>("sumOf").expect("This should work!").1
            )
        }

        #[test]
        fn can_parse_a_new_expression(){
            assert_eq!(
                Aggregation::new_with_limit(AggregationType::AvgOf, 3).unwrap(),
                parse_aggregation::<VerboseError<_>>("avgOf (3)").expect("This should work!").1
            )
        }

        #[test]
        fn can_parse_a_legacy_expression_star(){
            assert_eq!(
                Aggregation::new_no_limit(AggregationType::GAvgOf),
                parse_aggregation::<VerboseError<_>>("gAvgOf (*)").expect("This should work!").1
            )
        }

        #[test]
        fn can_parse_a_legacy_expression_limit_star(){
            assert_eq!(
                Aggregation::new_no_limit(AggregationType::GAvgOf),
                parse_aggregation::<VerboseError<_>>("gAvgOf limit(*)").expect("This should work!").1
            )
        }

        #[test]
        fn can_parse_a_legacy_expression_limit1(){
            assert_eq!(
                Aggregation::new_with_limit(AggregationType::GAvgOf, 99).unwrap(),
                parse_aggregation::<VerboseError<_>>("gAvgOf limit (99)").expect("This should work!").1
            )
        }

        #[test]
        fn can_parse_a_legacy_expression_limit2(){
            assert_eq!(
                Aggregation::new_with_limit(AggregationType::GAvgOf, 99).unwrap(),
                parse_aggregation::<VerboseError<_>>("gAvgOf limit(99)").expect("This should work!").1
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::voting::aggregations::{Aggregation, AggregationType};


    macro_rules! define_test {
        ($name: ident: $op: path, $expected: expr, $values: expr) => {
            #[test]
            fn $name(){
                let data = $values;
                let exp = $expected;
                let op = Aggregation::new_no_limit($op);
                let result1 = op.calculate_asc(data.clone().into_iter());
                let result2 = op.calculate_desc(data.into_iter());
                assert_eq!(exp, result1, "result1 {result1:?}");
                assert_eq!(exp, result2, "result2 {result2:?}");
            }
        };

        ($name: ident: $op: path, in $expected: expr, $values: expr) => {
            #[test]
            fn $name(){
                let data = $values;
                let exp = $expected;
                let op = Aggregation::new_no_limit($op);
                let result1 = op.calculate_asc(data.clone().into_iter());
                let result2 = op.calculate_desc(data.into_iter());
                println!("{result1:?}");
                println!("{result2:?}");
                assert!(exp.contains(&result1.expect("Fails to compute 1!")));
                assert!(exp.contains(&result2.expect("Fails to compute 2!")));
            }
        };

        ($name: ident: $op: path, limit $limit: expr, asc $expected1: expr, desc $expected2: expr, $values: expr) => {
            #[test]
            fn $name(){
                let data = $values;
                let exp1 = $expected1;
                let exp2 = $expected2;
                let op = Aggregation::new_with_limit($op, $limit).unwrap();
                let result1 = op.calculate_asc(data.clone().into_iter());
                let result2 = op.calculate_desc(data.into_iter());
                assert_eq!(exp1, result1, "result1 {result1:?}");
                assert_eq!(exp2, result2, "result2 {result2:?}");
            }
        };

        ($name: ident: $op: path, limit $limit: expr, asc in $expected1: expr, desc in $expected2: expr, $values: expr) => {
            #[test]
            fn $name(){
                let data = $values;
                let exp1 = $expected1;
                let exp2 = $expected2;
                let op = Aggregation::new_with_limit($op, $limit).unwrap();
                let result1 = op.calculate_asc(data.clone().into_iter());
                let result2 = op.calculate_desc(data.into_iter());
                assert!(exp1.contains(&result1.expect("Fails to compute!")), "failed result1 {result1:?}");
                assert!(exp2.contains(&result2.expect("Fails to compute!")), "failed result2 {result2:?}");
            }
        };
    }

    define_test! {
        can_calculate_the_sum:
        AggregationType::SumOf,
        Ok(45.),
        vec![1,2,3,4,5,6,7,8,9]
    }

    define_test! {
        can_calculate_the_max:
        AggregationType::MaxOf,
        Ok(10.),
        vec![1,2,10,3,4,5,6,7,8,9]
    }

    define_test! {
        can_calculate_the_min:
        AggregationType::MinOf,
        Ok(-10.),
        vec![1,2,-10,3,4,5,6,7,8,9]
    }

    define_test! {
        can_calculate_the_avg:
        AggregationType::AvgOf,
        Ok(5.5),
        vec![1,2,10,3,4,5,6,7,8,9]
    }

    define_test! {
        can_calculate_the_gavg:
        AggregationType::GAvgOf,
        in 4.5287286881..4.5287286882,
        vec![1,2,10,3,4,5,6,7,8,9]
    }




    define_test! {
        can_calculate_the_sum_lim:
        AggregationType::SumOf,
        limit 3usize,
        asc Ok(6.0),
        desc Ok(24.0),
        vec![4,5, 1,2,3,6,7,8,9]
    }

    define_test! {
        can_calculate_the_max_lim:
        AggregationType::MaxOf,
        limit 3usize,
        asc Ok(3.0),
        desc Ok(10.0),
        vec![1,2,10,3,4,8,5,6,9,7]
    }

    define_test! {
        can_calculate_the_min_lim:
        AggregationType::MinOf,
        limit 3usize,
        asc Ok(1.0),
        desc Ok(8.0),
        vec![1,2,10,3,4,8,5,6,9,7]
    }

    define_test! {
        can_calculate_the_avg_lim:
        AggregationType::AvgOf,
        limit 3usize,
        asc Ok(2.0),
        desc Ok(9.0),
        vec![1,2,10,3,4,8,5,6,9,7]
    }

    define_test! {
        can_calculate_the_gavg_lim:
        AggregationType::GAvgOf,
        limit 3usize,
        asc in 1.817120592..1.817120593f64,
        desc in 8.962809492..8.962809494f64,
        vec![1,2,10,3,4,5,6,7,8,9]
    }

}




