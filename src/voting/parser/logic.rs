#[allow(dead_code)]

use evalexpr::EvalexprError;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, char, digit1, multispace0, multispace1, one_of, space0, space1};
use nom::combinator::{cut, map, map_res, not, opt, peek, recognize};
use nom::error::{context, ContextError, ErrorKind, FromExternalError, ParseError};
use nom::{AsChar, InputIter, InputTakeAtPosition, IResult, Parser};
use nom::multi::{many1, many1_count};
use nom::sequence::{delimited, preceded, terminated, tuple};
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::voting::buildin::BuildInVoting;
use crate::voting::aggregations::parse::AggregationParserError;
use crate::voting::parser::structs::*;

const IMPORTANT_TOKENS: &str = "._+-*/%^=!<>&|,;: \"'";

const KW_ITER: &str = "foreach";
const KW_GLOBAL: &str = "global";
const KW_AGGREGATE: &str = "aggregate";
const KW_LET: &str = "let";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[derive(AsRefStr, Display, EnumString)]
pub enum Keyword {
    #[strum(serialize = "foreach")]
    ForEach,
    #[strum(serialize = "global")]
    Global,
    #[strum(serialize = "aggregate")]
    Aggregate,
    #[strum(serialize = "let")]
    Let
}

fn keyword<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, &str, E>
{
    context(
        "keyword",
        preceded(
            multispace0,
            alt((
                tag(KW_ITER),
                tag(KW_GLOBAL),
                tag(KW_AGGREGATE),
                tag(KW_LET),
            ))
        )
    )(input)
}


pub trait ErrorType<T>: ParseError<T> + ContextError<T> + FromExternalError<T, VotingParseError> + FromExternalError<T, AggregationParserError> {}
impl<C, T> ErrorType<T> for C where
    C:
    ParseError<T> +
    ContextError<T> +
    FromExternalError<T, VotingParseError> +
    FromExternalError<T, AggregationParserError>{}

#[derive(Debug, Clone, Error)]
pub enum VotingParseError {
    #[error("The if block is missing an expression")]
    IfExpressionMissing,
    #[error("The else block is missing, this is necessary for a statement!")]
    ElseBlockMissing,
    #[error("No Voting found!")]
    NoVotingFound,
    #[error("No expression or statement found!")]
    NoExpressionOrStatementFound,
    #[error(transparent)]
    EvalExpr(#[from] EvalexprError),
    #[error(transparent)]
    NotAKeyword(strum::ParseError),
    #[error("An empty index access does not work!")]
    EmptyIndexNotAllowed,
    #[error("An to range (..=) always needs a value after the =!")]
    ToRangeAlwaysNeedsValue
}



macro_rules! make_expr {
        ($vis:vis $name: ident, open=$open:literal, close=$close:literal, spacing= $space:ident, on_close_missing=$message: literal) => {
            $vis fn $name<I, O, E: ErrorType<I>, F>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
                where F: Parser<I, O, E>,
                      I: InputTakeAtPosition + nom::Slice<std::ops::RangeFrom<usize>> + InputIter + Clone,
                      <I as InputTakeAtPosition>::Item: AsChar + Clone,
                      <I as InputIter>::Item: AsChar
            {
                delimited(
                    preceded($space, char($open)),
                    preceded($space, inner),
                    context($message, cut(preceded($space, char($close)))),
                )
            }
        };
    }


make_expr!(
        s_expr,
        open='(',
        close=')',
        spacing= multispace0,
        on_close_missing="closing parentheses for single expr"
    );

make_expr!(
        s_expr_no_newline,
        open='(',
        close=')',
        spacing= space0,
        on_close_missing="closing parentheses for single expr (no newline)"
    );

make_expr!(
        b_exp,
        open='{',
        close='}',
        spacing= multispace0,
        on_close_missing="closing parentheses for block expr"
    );

// make_expr!(
//     c_exp,
//     open='[',
//     close=']',
//     spacing= multispace0,
//     on_close_missing="closing parentheses for tuple/array access"
// );

make_expr!(
        c_expr_no_newline,
        open='[',
        close=']',
        spacing= space0,
        on_close_missing="closing parentheses for tuple/array access (no newline)"
    );





pub fn variable_name<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E>
{
    context(
        "variable name",
        delimited(
            context("keyword check", peek(not(keyword))),
            nom::combinator::verify(
                recognize(
                    preceded(
                        peek(not(digit1)),
                        many1(alt((alphanumeric1, tag("_"))))
                    )
                ),
                |value: &str| value.len() > 0
            ),
            context("not name", peek(not(alt((alphanumeric1, tag("_"))))))
        )
    )(input)
}


fn voting_expression<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, VotingExpression, E> {
    fn collect_eval_expr<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
        context("single expression", recognize(
            many1_count(
                alt((
                    alphanumeric1,
                    s_expr_no_newline(collect_eval_expr),
                    recognize(one_of(IMPORTANT_TOKENS))
                ))
            )
        ))(input)
    }


    context(
        "expression",
        alt((
            voting_get_tuple_expression,
            map(
                preceded(multispace0, inner_if_else),
                VotingExpression::IfElse
            ),
            map_res(
                preceded(multispace0, collect_eval_expr),
                |value| {
                    VotingExpression::parse_as_single(value).map_err(VotingParseError::EvalExpr)
                }
            ),
        ))
    )(input)
}

fn voting_get_tuple_expression<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, VotingExpression, E> {
    context(
        "get tuple",
        map(
            tuple((
                preceded(multispace0, variable_name),
                preceded(space0, c_expr_no_newline(parse_index_or_range))
            )),
            |(name, idx)| {
                VotingExpression::TupleGet {
                    idx,
                    variable_name: name.to_string()
                }
            }
        )
    )(input)
}

fn parse_index_or_range<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, IndexOrRange, E> {
    context(
        "parse index/range",
        map_res(
            tuple((
                opt(digit1),
                opt(
                    tuple((
                        preceded(space0, tag::<&str, &str, E>("..")),
                        opt(tag("=")),
                        opt(preceded(space0, digit1))
                    ))
                ),
            )),
            |(first, dots_and_second)| {
                if let Some((_, eq, second)) = dots_and_second {
                    if eq.is_some() {
                        if let Some(second) = second {
                            if let Some(first) = first {
                                Ok(IndexOrRange::RangeInclusive(first.parse().unwrap()..=second.parse().unwrap()))
                            } else {
                                Ok(IndexOrRange::RangeToInclusive(..=second.parse().unwrap()))
                            }
                        } else {
                            Err(VotingParseError::ToRangeAlwaysNeedsValue)
                        }
                    } else {
                        if let Some(second) = second {
                            if let Some(first) = first {
                                Ok(IndexOrRange::Range(first.parse().unwrap() .. second.parse().unwrap()))
                            } else {
                                Ok(IndexOrRange::RangeTo( ..second.parse().unwrap()))
                            }
                        } else {
                            if let Some(first) = first {
                                Ok(IndexOrRange::RangeFrom(first.parse().unwrap()..))
                            } else {
                                Ok(IndexOrRange::RangeFull)
                            }
                        }
                    }
                } else {
                    if let Some(value) = first {
                        Ok(IndexOrRange::Index(value.parse().unwrap()))
                    } else {
                        Err(VotingParseError::EmptyIndexNotAllowed)
                    }
                }
            }
        )
    )(input)
}



fn voting_list<'a, E: ErrorType<&'a str>>(inner: &'a str) -> IResult<&'a str, VotingExecutableList, E>  {
    context(
        "voting list",
        preceded(multispace0, alt((
            b_exp(map_res(
                many1(preceded(multispace0, voting_or_statement)),
                |value|
                    VotingExecutableList::pack_vec(value).ok_or(VotingParseError::NoExpressionOrStatementFound)
            )),
            map(voting_or_statement, VotingExecutableList::pack_single)
        )))
    )(inner)
}

fn parse_if<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, (VotingExpression, VotingExecutableList), E> {
    context(
        "parse if",
        preceded(
            preceded(multispace0, tag("if")),
            tuple((
                s_expr(voting_expression),
                voting_list,
            ))
        )
    )(input)
}

fn inner_if_else<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, InnerIfElse, E> {
    context(
        "parse if else",
        map(
            tuple((
                parse_if,
                preceded(
                    preceded(multispace0, tag("else")),
                    voting_list
                )
            )),
            InnerIfElse::from_expr
        )
    )(input)
}

fn voting_statement<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, VotingStatement, E> {
    context(
        "statement",
        alt((
            map(delimited(multispace0, parse_if, not(preceded(multispace0, tag("else")))), |(cond, if_block)| {
                VotingStatement::If {
                    cond,
                    if_block
                }
            }),
            map(
                context(
                    "set variable",
                    tuple((
                        delimited(
                            delimited(multispace0, tag(KW_LET), space1),
                            variable_name,
                            preceded(space0, char('='))
                        ),
                        preceded(space0, voting_list)
                    ))
                ),
                |(name, expression)| {
                    VotingStatement::SetVariable {
                        variable_name: name.to_string(),
                        expression
                    }
                }
            )
        ))
    )(input)
}

fn voting_or_statement<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, VotingExpressionOrStatement, E> {
    context(
        "voting or statement",
        alt((
            map(voting_statement, VotingExpressionOrStatement::pack_stmt),
            map(voting_expression, VotingExpressionOrStatement::pack_expr),
        ))
    )(input)
}


fn voting_operation<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, VotingOperation, E> {
    preceded(
        multispace0,
        alt((
            preceded(
                terminated(
                    tag(KW_ITER),
                    preceded(multispace0, tag(":"))
                ),
                preceded(
                    multispace1,
                    map(voting_list, |expr| VotingOperation::IterScope {expr})
                )
            ),
            preceded(
                terminated(
                    tag(KW_GLOBAL),
                    preceded(multispace0, tag(":"))
                ),
                preceded(
                    multispace1,
                    map(voting_list, |expr| VotingOperation::GlobalScope {expr})
                )
            ),
            preceded(
                tag(KW_AGGREGATE),
                map(
                    tuple((
                        terminated(
                            s_expr(
                                tuple((
                                    delimited(
                                        multispace0,
                                        variable_name,
                                        preceded(multispace0, tag("="))
                                    ),
                                    preceded(
                                        multispace0,
                                        crate::voting::aggregations::parse::parse_aggregation
                                    )
                                ))
                            ),
                            preceded(multispace0, tag(":"))
                        ),
                        preceded(multispace0, voting_list)
                    )),
                    |((variable_name, op), expr)|
                        VotingOperation::AggregationScope {
                            variable_name: variable_name.to_string(),
                            op,
                            expr
                        }
                )
            )
        ))
    )(input)
}

fn voting_function<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, VotingFunction, E> {
    map_res(many1(preceded(multispace0, voting_operation)), |mut value| {
        match value.len() {
            0 => Err(VotingParseError::NoVotingFound),
            1 => Ok(VotingFunction::Single(value.swap_remove(0))),
            _ => Ok(VotingFunction::Multi(value))
        }
    })(input)
}

struct BuildInVotingParser;

impl<Input, Error: ParseError<Input>> Parser<Input, BuildInVoting, Error> for BuildInVotingParser
    where Input: InputTakeAtPosition + AsRef<str>,
          <Input as InputTakeAtPosition>::Item: AsChar,
{
    fn parse(&mut self, input: Input) -> IResult<Input, BuildInVoting, Error> {
        let (outp,to_parse) = input.split_at_position1_complete(|item| !item.is_alphanum(), ErrorKind::AlphaNumeric)?;
        match to_parse.as_ref().parse() {
            Ok(value) => {
                Ok((outp, value))
            }
            Err(_) => {
                Err(nom::Err::Error(Error::from_error_kind(input, ErrorKind::Tag)))
            }
        }
    }
}

fn build_in_voting<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, BuildInVoting, E> {
    preceded(
        multispace0,
        BuildInVotingParser
    )(input)
}

// fn voting<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, Voting<dyn VotingMethod>, E> {
//
// }

#[cfg(test)]
mod test {
    use evalexpr::{ContextWithMutableVariables, HashMapContext};
    use nom::{Finish};
    use nom::error::VerboseError;
    use crate::voting::{VotingMethod};
    use crate::voting::parser::logic::voting_function;

    const TEST: &str = "aggregate(sss = sumOf): {
            let katze = if (a+b == (c+d)) {
                r = true
                x = -3 + 4 + c * 2
                z = (true, -1, (3), false)
                let _temp = z[1]
                o = -_temp
                y = -(a + b)
                value = 9 - 2 + d * x
                pp
            } else {
                r = true
                x = 9 + 7 + a
                z = (true, -2, (3), false)
                let _temp = z[1]
                o = -_temp
                value = (8 + 7) * b + 1
                pp + 1
            }
            let katze = katze + 1
            katze
        }";

    #[test]
    fn versuch(){
        let mut conbtext = HashMapContext::new();
        conbtext.set_value("a".to_string(), 3.into()).unwrap();
        conbtext.set_value("b".to_string(), 2.into()).unwrap();
        conbtext.set_value("c".to_string(), 1.into()).unwrap();
        conbtext.set_value("d".to_string(), 4.into()).unwrap();

        let result= voting_function::<VerboseError<_>>(TEST).finish();
        match &result {
            Ok(value) => {
                println!("{}", value.1);
            }
            Err(value) => {
                println!("{}", value.to_string());
            }
        }
        // println!("{}", result.unwrap_err().to_string());
        // println!("{}", result.unwrap_err().to_string());
        let mut x = vec![
            HashMapContext::new(),
            HashMapContext::new(),
            HashMapContext::new(),
        ];

        for (i, z) in x.iter_mut().enumerate() {
            z.set_value("pp".to_string(), (i as i64).into()).unwrap()
        }

        println!("{:?}", result.unwrap().1.execute(&mut conbtext, &mut x));
        println!("{:?}", conbtext);
        println!("{:?}", x);
    }
}