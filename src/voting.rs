use evalexpr::{ContextWithMutableVariables, EvalexprError, EvalexprResult, FloatType, HashMapContext, Value};
use itertools::{FoldWhile, Itertools, Position};
use thiserror::Error;
use crate::toolkit::evalexpr::{CombineableContext, CombinedContextWrapper};
use crate::voting::aggregations::{Aggregation, BulkOperationError};

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
    Bulk(#[from] BulkOperationError)
}

type VResult<T> = Result<T, VotingExpressionError>;


enum VotingFunction {
    Single(VotingScopes),
    Multi(Vec<VotingScopes>)
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

enum VotingScopes {
    /// for each { <expr> \n <expr> } || for each <expr>
    IterScope {
        expr: VotingExpressions
    },
    /// global { <expr> \n <expr> } || <expr>
    GlobalScope {
        expr: VotingExpressions
    },
    /// <variable_name> = <aggregation> = { <expr> \n <expr> } || <variable_name> = <aggregation> <expr>
    AggregationScope {
        variable_name: String,
        op: Aggregation,
        expr: VotingExpressions
    }
}



impl VotingScopes {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VResult<Value>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables
    {
        match self {
            VotingScopes::IterScope {
                expr
            } => {
                for value in voters {
                    expr.execute(&mut value.combine_with(global_context))?;
                }
                Ok(Value::Empty)
            }
            VotingScopes::AggregationScope {
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

            VotingScopes::GlobalScope { expr } => {
                Ok(expr.execute(global_context)?)
            }
        }
    }
}

type Context<'a, A, B> = CombinedContextWrapper<'a, A, B>;


enum VotingExpressions {
    Single(VotingExpression),
    Multi(Vec<VotingExpression>),
}

impl VotingExpressions {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value>
    {
        match self {
            VotingExpressions::Single(value) => {
                value.execute(context)
            }
            VotingExpressions::Multi(values) => {
                values.iter().fold_while(
                    Ok(Value::Empty),
                    |value, expr| {
                        let result = expr.execute(context);
                        if result.is_ok() {
                            FoldWhile::Continue(result)
                        } else {
                            FoldWhile::Done(result)
                        }
                    }
                ).into_inner()
            }
        }
    }
}

enum VotingExpression {
    Expression {
        expr: evalexpr::Node,
    },
    /// if (<condition>) { <if_true_expr> } else { <if_false_expr> }
    IfElseStatement {
        condition: evalexpr::Node,
        if_true_expr: Box<VotingExpressions>,
        if_false_expr: Box<VotingExpressions>,
    },
    /// <variable> = if (<condition>) { <if_true_expr> } else { <if_false_expr> }
    IfElseExpression {
        variable_name: String,
        condition: evalexpr::Node,
        if_true_expr: Box<VotingExpressions>,
        if_false_expr: Box<VotingExpressions>,
    },
}



impl VotingExpression {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VResult<Value>
    {
        match self {
            VotingExpression::IfElseStatement { condition, if_true_expr, if_false_expr } => {
                if condition.eval_boolean_with_context(context)? {
                    if_true_expr.execute(context)?;
                } else {
                    if_false_expr.execute(context)?;
                }
                Ok(Value::Empty)
            }
            VotingExpression::Expression { expr } => {
                Ok(expr.eval_with_context_mut(context)?)
            }
            VotingExpression::IfElseExpression {
                condition,
                if_false_expr,
                if_true_expr,
                variable_name
            } => {

                let result = if condition.eval_boolean_with_context(context)? {
                    if_true_expr.execute(context)?
                } else {
                    if_false_expr.execute(context)?
                };
                context.set_value(variable_name.clone(), result)?;
                Ok(Value::Empty)
            }
        }
    }
}

mod parse {
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_till1};
    use nom::character::complete::{alphanumeric1, anychar, space1};
    use nom::IResult;
    use nom::multi::many1;
    use nom::sequence::{delimited, preceded, tuple};
    use crate::toolkit::nom::ws;

    pub fn parse_voting_expression(input: &str) -> IResult<&str, ()>{
        alt((
            preceded(
                tag("if"),
                tuple((
                    ws(delimited(
                        tag("("),
                        many1(anychar),
                        tag(")")
                    )),

                ))
            )
        ))(input)
    }
}
