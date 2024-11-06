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

use std::num::{ParseIntError};
use evalexpr::EvalexprError;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, char, digit1, multispace0, multispace1, one_of, space0, space1};
use nom::combinator::{cut, map, map_res, not, opt, peek, recognize};
use nom::error::{context, ContextError, ErrorKind, FromExternalError, ParseError};
use nom::{AsChar, InputIter, InputTakeAtPosition, IResult, Parser};
use nom::multi::{many1, many1_count};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::variable_provider::variable_names::{reserved_variable_name};
use crate::voting::buildin::BuildInVoting;
use crate::voting::aggregations::parse::AggregationParserError;
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::logic::VotingParseError::{NoRegistryProvided, NoVotingInRegistryFound, UnableToParseInt};
use crate::voting::parser::voting_function::*;
use crate::voting::parser::voting_function::VotingExecution::Limited;
use crate::voting::VotingWithLimit;

const IMPORTANT_TOKENS: &str = "._+-*/%^=!<>&|,;: \"'";

const KW_ITER: &str = "foreach";
const KW_GLOBAL: &str = "global";
const KW_AGGREGATE: &str = "aggregate";
const KW_EXECUTE: &str = "execute";
const KW_LET: &str = "let";
const KW_DECLARE: &str = "declare";

/// Keywords known to the parser
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[derive(AsRefStr, Display, EnumString)]
pub enum Keyword {
    #[strum(serialize = "foreach")]
    ForEach,
    #[strum(serialize = "global")]
    Global,
    #[strum(serialize = "aggregate")]
    Aggregate,
    #[strum(serialize = "call")]
    Call,
    #[strum(serialize = "let")]
    Let
}

fn let_variable_declaration<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, ParserInput<'a,'b>,  E> {
    preceded(
        delimited(multispace0, tag(KW_LET), space1),
        preceded(
            context("special variable check failed", peek(not(reserved_variable_name))),
            variable_name,
        )
    )(input)
}

fn keyword<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, ParserInput<'a,'b>,  E>
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
                tag(KW_EXECUTE),
                tag(KW_DECLARE),
            ))
        )
    )(input)
}

/// The errortype used by the parser
pub trait ErrorType<T>: ParseError<T> + ContextError<T> + FromExternalError<T, VotingParseError> + FromExternalError<T, AggregationParserError> {}
impl<C, T> ErrorType<T> for C where
    C:
    ParseError<T> +
    ContextError<T> +
    FromExternalError<T, VotingParseError> +
    FromExternalError<T, AggregationParserError>{}

/// Errors when parsing a string to a voting.
#[derive(Debug, Clone, Error)]
pub enum VotingParseError {
    // #[error("The if block is missing an expression")]
    // IfExpressionMissing,
    // #[error("The else block is missing, this is necessary for a statement!")]
    // ElseBlockMissing,
    #[error("No Voting found!")]
    NoVotingFound,
    #[error("No expression or statement found!")]
    NoExpressionOrStatementFound,
    #[error(transparent)]
    EvalExpr(#[from] EvalexprError),
    // #[error(transparent)]
    // NotAKeyword(strum::ParseError),
    #[error("An empty index access does not work!")]
    EmptyIndexNotAllowed,
    #[error("An to range (..=) always needs a value after the =!")]
    ToRangeAlwaysNeedsValue,
    #[error("No voting with the name {0} in the registy found!")]
    NoVotingInRegistryFound(String),
    #[error("There was no registy provided!")]
    NoRegistryProvided,
    #[error(transparent)]
    UnableToParseInt(#[from] ParseIntError)
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
    ($vis:vis $name: ident, open=$open:literal, close=$close:literal, on_close_missing=$message: literal) => {
        $vis fn $name<I, O, E: ErrorType<I>, F>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
            where F: Parser<I, O, E>,
                  I: InputTakeAtPosition + nom::Slice<std::ops::RangeFrom<usize>> + InputIter + Clone,
                  <I as InputTakeAtPosition>::Item: AsChar + Clone,
                  <I as InputIter>::Item: AsChar
        {
            delimited(
                preceded($space, char($open)),
                inner,
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
    pub(crate) b_exp,
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





pub(crate) fn variable_name<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, ParserInput<'a,'b>,  E>
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
                |value: &ParserInput| value.len() > 0
            ),
            context("not name", peek(not(alt((alphanumeric1, tag("_"))))))
        )
    )(input)
}


fn voting_expression<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingExpression, E> {
    fn collect_eval_expr<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, ParserInput<'a,'b>,  E> {
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

fn voting_get_tuple_expression<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingExpression, E> {
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

fn parse_index_or_range<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, IndexOrRange, E> {
    context(
        "parse index/range",
        map_res(
            tuple((
                opt(digit1),
                opt(
                    tuple((
                        preceded(space0, tag::<&str, ParserInput, E>("..")),
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
                                Ok(IndexOrRange::RangeTo(..second.parse().unwrap()))
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



fn voting_list<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingExecutableList, E>  {
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
    )(input)
}

fn parse_if<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, (VotingExpression, VotingExecutableList), E> {
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

fn inner_if_else<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, InnerIfElse, E> {
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

fn voting_statement<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingStatement, E> {
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
                        terminated(
                            let_variable_declaration,
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

fn voting_or_statement<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingExpressionOrStatement, E> {
    context(
        "voting or statement",
        alt((
            map(voting_statement, VotingExpressionOrStatement::pack_stmt),
            map(voting_expression, VotingExpressionOrStatement::pack_expr),
        ))
    )(input)
}


pub(crate) fn parse_limited<'a, 'b, O1, O2, E: ErrorType<ParserInput<'a,'b>>, F>(
    parser: F
) -> impl Parser<ParserInput<'a,'b>, VotingWithLimit<O2>, E> where F: Parser<ParserInput<'a,'b>, O1, E>, O1: Into<O2> {
    map(
        separated_pair(
            parser,
            space0,
            map_res(s_expr_no_newline(digit1), |value: ParserInput| {
                let x: &str = value.as_ref();
                match x.parse() {
                    Ok(value) => {
                        Ok(value)
                    }
                    Err(err) => {
                        Err(UnableToParseInt(err))
                    }
                }
            })
        ),
        |(voting, value)| {
            VotingWithLimit::new(value, voting.into())
        }
    )
}

fn voting_execution<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingExecution, E> {
    fn simple_voting_execution<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingExecution, E> {
        alt((
            map(build_in_voting, VotingExecution::BuildIn),
            map_res(
                preceded(space0, variable_name),
                |value| {
                    match value.registry() {
                        None => {Err(NoRegistryProvided)}
                        Some(registry) => {
                            match registry.get(value.as_ref()) {
                                None => {
                                    Err(NoVotingInRegistryFound(value.to_string()))
                                }
                                Some(voting) => {
                                    Ok(VotingExecution::Parsed(value.to_string(), voting))
                                }
                            }
                        }
                    }
                }
            )
        ))(input)
    }

    alt((
        map(
            parse_limited(simple_voting_execution),
            |voting| { Limited(voting) }
        ),
        simple_voting_execution
    ))(input)
}


fn voting_operation<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingOperation, E> {
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
                                    terminated(
                                        let_variable_declaration,
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
            ),
            preceded(
                tag(KW_EXECUTE),
                map(
                    terminated(
                        s_expr_no_newline(
                            tuple((
                                terminated(
                                    let_variable_declaration,
                                    preceded(space0, tag("="))
                                ),
                                preceded(
                                    space0,
                                    voting_execution
                                )
                            ))
                        ),
                        preceded(space0, char(';'))
                    ),
                    |(variable_name, call)| {
                        VotingOperation::Execute {
                            variable_name: variable_name.to_string(),
                            execution: call
                        }
                    }
                )
            )
        ))
    )(input)
}


fn voting_function_provider<'a,'b, E: ErrorType<ParserInput<'a,'b>>>(is_root: bool) -> impl FnMut(ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingFunction, E>{
    map_res(many1(preceded(multispace0, voting_operation)), move |mut value| {
        match value.len() {
            0 => Err(VotingParseError::NoVotingFound),
            1 => Ok(VotingFunction::Single(value.swap_remove(0), is_root)),
            _ => Ok(VotingFunction::Multi(value))
        }
    })
}

/// Parses a voting function
fn voting_function<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingFunction, E> {
    voting_function_provider(false)(input)
}

pub(crate) fn global_voting_function<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingFunction, E> {
    alt((
        b_exp(voting_function_provider(true)),
        voting_function_provider(false),
    ))(input)
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

pub(crate) fn build_in_voting<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a, 'b>) -> IResult<ParserInput<'a,'b>, BuildInVoting, E> {
    preceded(
        multispace0,
        BuildInVotingParser
    )(input)
}

pub(crate) fn voting<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingAndName, E> {
    fn voting_impl<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, VotingAndName, E> {
        map(
            pair(
                delimited(
                    delimited(multispace0, tag(KW_DECLARE), space1),
                    preceded(
                        context("special variable check failed", peek(not(reserved_variable_name))),
                        variable_name,
                    ),
                    multispace1
                ),
                b_exp(voting_function)
            ),
            VotingAndName::from
        )(input)
    }

    alt((
        b_exp(voting_impl),
        voting_impl
    ))(input)
}



#[cfg(test)]
mod test {
    use evalexpr::{ContextWithMutableVariables, HashMapContext};
    use nom::{Finish};
    use nom::error::VerboseError;
    use crate::variable_provider::variable_names::{NUMBER_OF_VOTERS, SCORE};
    use crate::voting::{VotingMethod};
    use crate::voting::parser::logic::{b_exp, voting, voting_function};

    const TEST: &str = "aggregate(let sss = sumOf): {
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

    const TEST2: &str = "
    declare my_voting {
        execute(let the_votes_selected = Voters);
        aggregate(let sss = sumOf): {
            let katze = if (a+b == (c+d)) {
                r = true
                x = -3 + 4 + c * 2
                z = (true, -1, (3), false)
                let _temp = z[1]
                o = -_temp
                y = -(a + b)
                value = 9 - 2 + d * x
                pp + 2
            } else {
                r = true
                x = 9 + 7 + a
                z = (true, -2, (3), false)
                let _temp = z[1]
                o = -_temp
                value = (8 + 7) * b + 1
                pp + 3
            }
            let katze = katze + the_votes_selected
            katze
        }
    }
    ";

    const TEST3: &str = "
    {
        aggregate(let sss = sumOf): {score}
        global: sss
    }
    ";

    #[test]
    fn can_parse(){
        let x = b_exp::<_, _, VerboseError<_>, _>(voting_function)(TEST3.into());
        let x = x.unwrap();
        println!("{}", x.1)
    }

    #[test]
    fn versuch(){
        let mut conbtext = HashMapContext::new();
        conbtext.set_value("a".to_string(), 3.into()).unwrap();
        conbtext.set_value("b".to_string(), 2.into()).unwrap();
        conbtext.set_value("c".to_string(), 1.into()).unwrap();
        conbtext.set_value("d".to_string(), 4.into()).unwrap();

        let result= voting_function::<VerboseError<_>>(TEST.into()).finish();
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

    #[test]
    fn versuch2(){
        let mut conbtext = HashMapContext::new();
        conbtext.set_value("a".to_string(), 3.into()).unwrap();
        conbtext.set_value("b".to_string(), 2.into()).unwrap();
        conbtext.set_value("c".to_string(), 1.into()).unwrap();
        conbtext.set_value("d".to_string(), 4.into()).unwrap();
        conbtext.set_value(NUMBER_OF_VOTERS.to_string(), 4.into()).unwrap();

        let result= voting::<VerboseError<_>>(TEST2.into()).finish();
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
            z.set_value("pp".to_string(), (i as i64).into()).unwrap();
            z.set_value(SCORE.to_string(), (0.3 * i as f64).into()).unwrap();
        }

        println!("{:?}", result.unwrap().1.1.execute(&mut conbtext, &mut x));
        println!("{:?}", conbtext);
        println!("{:?}", x);
    }
}