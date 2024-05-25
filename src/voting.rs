use std::fmt::{Debug, Display, Formatter, write};
use evalexpr::{ContextWithMutableVariables, EvalexprError, EvalexprResult, Value};
use itertools::{FoldWhile, Itertools, Position};
use thiserror::Error;
use crate::toolkit::evalexpr::CombineableContext;
use crate::voting::aggregations::{Aggregation, AggregationError};

mod parser;
mod aggregations;


pub struct Voting {
    name: String,
    expr: VotingFunction
}

#[derive(Debug, Error)]
pub enum VotingExpressionError {
    #[error(transparent)]
    Eval(#[from] EvalexprError),
    #[error(transparent)]
    Agg(#[from] AggregationError)
}

type VResult<T> = Result<T, VotingExpressionError>;

#[derive(Debug)]
enum VotingFunction {
    Single(VotingOperation),
    Multi(Vec<VotingOperation>)
}

impl VotingFunction {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VResult<Value>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables
    {
        match self {
            VotingFunction::Single(value) => {
                value.execute(global_context, voters)
            }
            VotingFunction::Multi(values) => {
                for (pos, expr) in values.iter().with_position() {
                    match pos {
                        Position::First | Position::Middle => {
                            expr.execute(global_context, voters)?;
                        }
                        Position::Last | Position::Only => {
                            return expr.execute(global_context, voters)
                        }
                    }
                }
                unreachable!()
            }
        }
    }
}


/// The operation beeing executed
#[derive(Debug)]
enum VotingOperation {
    /// foreach: { <expr> \n <expr> } || for each <expr>
    IterScope {
        expr: VotingExecutableList
    },
    /// global: { <expr> \n <expr> } || <expr>
    GlobalScope {
        expr: VotingExecutableList
    },
    /// aggregate(<variable_name> = <aggregation>): { <expr> \n <expr> } || <variable_name> = <aggregation> <expr>
    AggregationScope {
        variable_name: String,
        op: Aggregation,
        expr: VotingExecutableList
    }
}

impl VotingOperation {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VResult<Value>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables
    {
        match self {
            VotingOperation::IterScope {
                expr
            } => {
                for value in voters {
                    expr.execute(&mut value.combine_with(global_context))?;
                }
                Ok(Value::Empty)
            }
            VotingOperation::AggregationScope {
                variable_name,
                op,
                expr
            } => {
                let value = voters
                    .into_iter()
                    .map(|value|
                        expr
                        .execute(&mut value.combine_with(global_context))
                        .and_then(|value| value.as_number().map_err(|op| op.into())))
                    .collect::<Result<Vec<_>, _>>()?;
                let new_result = op.calculate_desc(value.into_iter())?;
                global_context.set_value(variable_name.clone(), new_result.into())?;
                Ok(new_result.into())
            }

            VotingOperation::GlobalScope { expr } => {
                Ok(expr.execute(global_context)?)
            }
        }
    }
}

trait VotingExecutable {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value>;

}

#[derive(Debug)]
enum VotingExecutableList {
    Single(Box<VotingExpressionOrStatement>),
    Multiple(Vec<VotingExpressionOrStatement>)
}

impl VotingExecutableList {
    fn pack_single(expr: VotingExpressionOrStatement) -> Self {
        Self::Single(expr.into())
    }

    fn pack_vec(mut values: Vec<VotingExpressionOrStatement>) -> Option<Self> {
        match values.len() {
            0 => None,
            1 => Some(Self::Single(Box::new(values.swap_remove(0)))),
            _ => Some(Self::Multiple(values))
        }
    }
}

impl VotingExecutable for VotingExecutableList {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value> {
        match self {
            VotingExecutableList::Single(value) => {
                value.execute(context)
            }
            VotingExecutableList::Multiple(values) => {
                values.iter().fold_while(Ok(Value::Empty), |_, value| {
                    match value.execute(context) {
                        a @Ok(_) => {FoldWhile::Continue(a)}
                        b @Err(_) => {FoldWhile::Done(b)}
                    }
                }).into_inner()
            }
        }
    }
}

/// An if else expression or statement.
#[derive(Debug)]
struct InnerIfElse {
    cond: Box<VotingExpression>,
    if_block: VotingExecutableList,
    else_block: VotingExecutableList,
}

impl InnerIfElse {
    #[inline]
    pub fn new(cond: VotingExpression, if_block: VotingExecutableList, else_block: VotingExecutableList) -> Self {
        Self {
            cond: cond.into(),
            if_block,
            else_block
        }
    }

    fn from_expr(((cond, if_block), else_block): ((VotingExpression, VotingExecutableList), VotingExecutableList)) -> Self {
        Self::new(cond, if_block, else_block)
    }
}

impl VotingExecutable for InnerIfElse {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value> {
        if self.cond.execute(context)?.as_boolean()? {
            self.if_block.execute(context)
        } else {
            self.else_block.execute(context)
        }
    }
}


/// Either a statement or a expression
#[derive(Debug)]
enum VotingExpressionOrStatement {
    Expression {
        expr: VotingExpression
    },
    Statement {
        stmt: Box<VotingStatement>
    }
}

impl VotingExpressionOrStatement {
    pub fn pack_expr(expr: VotingExpression) -> Self {
        Self::Expression {expr}
    }
    pub fn pack_stmt(stmt: VotingStatement) -> Self {
        Self::Statement {stmt: stmt.into()}
    }
}

impl VotingExecutable for VotingExpressionOrStatement {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value>
    {
        match self {
            VotingExpressionOrStatement::Expression{expr} => {
                expr.execute(context)
            }
            VotingExpressionOrStatement::Statement{stmt} => {
                stmt.execute(context)
            }
        }
    }
}

/// The statements that can be used inside of votings
#[derive(Debug)]
enum VotingStatement {
    If {
        cond: VotingExpression,
        if_block: VotingExecutableList,
    },
    SetVariable {
        variable_name: String,
        expression: VotingExecutableList
    }
}

impl VotingExecutable for VotingStatement {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value>
    {
        match self {
            VotingStatement::If { cond, if_block } => {
                if cond.execute(context)?.as_boolean()? {
                    if_block.execute(context)?;
                }
            }
            VotingStatement::SetVariable {
                variable_name,
                expression
            } => {
                let res = expression.execute(context)?;
                context.set_value(variable_name.clone(), res)?;

            }
        }
        Ok(Value::Empty)
    }
}

/// An expression or multiple expressions
enum VotingExpression {
    Expr(evalexpr::Node),
    IfElse(InnerIfElse)
}

impl VotingExpression {
    #[inline(always)]
    fn parse_as_single(s: &str) -> EvalexprResult<Self> {
        Ok(VotingExpression::Expr(evalexpr::build_operator_tree(s)?))
    }
}


impl VotingExecutable for VotingExpression {
    #[inline(always)]
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value>
    {
        match self {
            VotingExpression::Expr(value) => {
                Ok(value.eval_with_context_mut(context)?)
            }
            VotingExpression::IfElse(value) => {
                value.execute(context)
            }
        }
    }
}

impl Debug for VotingExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VotingExpression::Expr(value) => {f.write_str(&value.to_string())}
            VotingExpression::IfElse(value) => {
                f.debug_struct("IfElse").field("if_else", value).finish()
            }
        }
    }
}

pub(crate) mod parse {
    use std::ops::RangeTo;
    use evalexpr::EvalexprError;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{alpha1, alphanumeric0, alphanumeric1, char, digit1, multispace0, multispace1, one_of, space0, space1};
    use nom::combinator::{cut, map, map_res, not, peek, recognize};
    use nom::error::{context, ContextError, FromExternalError, ParseError};
    use nom::{AsChar, Compare, error, InputIter, InputLength, InputTake, InputTakeAtPosition, IResult, Offset, Parser, Slice};
    use nom::multi::{count, many1, many1_count};
    use nom::sequence::{delimited, preceded, terminated, tuple};
    use thiserror::Error;
    use crate::toolkit::nom::ws;
    use crate::voting::{InnerIfElse, VotingExecutableList, VotingExpression, VotingExpressionOrStatement, VotingFunction, VotingOperation, VotingStatement};
    use crate::voting::aggregations::parse::AggregationParserError;
    use crate::voting::parse::VotingParseError::NoVotingFound;

    const IMPORTANT_TOKENS: &str = "+-*/%^=!<>&|,;";

    const KW_ITER: &str = "foreach";
    const KW_GLOBAL: &str = "global";
    const KW_AGGREGATE: &str = "aggregate";

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

    fn is_a_keyword<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E>
    {

        context(
            "not a keyword check",
            peek(
                preceded(
                    multispace0,
                    alt((
                        tag(KW_ITER),
                        tag(KW_GLOBAL),
                        tag(KW_AGGREGATE),
                    ))
                )
            )
        )(input)
    }



    pub fn variable_name<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E>
    {
        context(
            "variable name",
            delimited(
                context("keyword check", not(is_a_keyword)),
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
        on_close_missing="closing parentheses for single expr"
    );

    make_expr!(
        b_exp,
        open='{',
        close='}',
        spacing= multispace0,
        on_close_missing="closing parentheses for block expr"
    );



    fn voting_expression<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, VotingExpression, E> {
        fn collect_eval_expr<'a, E: ErrorType<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
            context("single expression", recognize(
                many1_count(
                    alt((
                        alphanumeric1,
                        s_expr_no_newline(collect_eval_expr),
                        recognize(one_of("_+-*/%^=!<>&|,; "))
                    ))
                )
            ))(input)
        }


        context(
            "expression",
            alt((
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
                            delimited(multispace0, variable_name, preceded(multispace0, char('='))),
                            preceded(multispace0, voting_list)
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
                0 => Err(NoVotingFound),
                1 => Ok(VotingFunction::Single(value.swap_remove(0))),
                _ => Ok(VotingFunction::Multi(value))
            }
        })(input)
    }



    #[cfg(test)]
    mod test {
        use evalexpr::{ContextWithMutableVariables, HashMapContext};
        use nom::{Finish};
        use nom::error::VerboseError;
        use crate::voting::parse::{variable_name, voting_function, voting_list};
        use crate::voting::{VotingExecutable, VotingFunction};

        const TEST: &str = "aggregate(sss = sumOf): {
            if (a+b == (c+d)) {
                r = true
                x = 3 + 4 + c * 2
                9 - 2 + d * x
                pp
            } else {
                r = true
                x = 9 + 7 + a
                (8 + 7) * b + 1
                pp + 1
            }
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
                    println!("{:?}", value);
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
}


