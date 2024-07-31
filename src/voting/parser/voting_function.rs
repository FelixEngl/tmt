use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::{Deref, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::sync::Arc;
use evalexpr::{ContextWithMutableVariables, EvalexprError, EvalexprResult, Node, TupleType, Value};
use itertools::{FoldWhile, Itertools, Position};
use crate::toolkit::evalexpr::CombineableContext;
use crate::voting::{BuildInVoting, VotingExpressionError, VotingMethod, VotingMethodContext, VotingMethodMarker, VotingResult, VotingWithLimit};
use crate::voting::aggregations::Aggregation;
use crate::voting::display::{DisplayTree, IndentWriter};
use crate::voting::parser::traits::VotingExecutable;
use crate::voting::traits::LimitableVotingMethodMarker;
use crate::voting::walk::walk_left_to_right;
use crate::voting::display::impl_display_for_displaytree;
use crate::voting::parser::input::ParserInput;


/// A tuple with a name and a voting function
#[derive(Debug, Clone)]
pub struct VotingAndName(pub String, pub VotingFunction);

impl From<(String, VotingFunction)> for VotingAndName {
    fn from(value: (String, VotingFunction)) -> Self {
        Self(value.0, value.1)
    }
}

impl From<(&str, VotingFunction)> for VotingAndName {
    fn from(value: (&str, VotingFunction)) -> Self {
        Self(value.0.to_string(), value.1)
    }
}

impl<'a, 'b> From<(ParserInput<'a, 'b>, VotingFunction)> for VotingAndName {
    fn from(value: (ParserInput<'a, 'b>, VotingFunction)) -> Self {
        Self(value.0.to_string(), value.1)
    }
}

impl DisplayTree for VotingAndName {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        write!(f, "declare {} ", self.0)?;
        write!(f, "{{")?;
        f.indent(2);
        write!(f, "\n")?;
        DisplayTree::fmt(&self.1, f)?;
        f.dedent(2);
        write!(f, "\n")?;
        write!(f, "}}")
    }
}

impl_display_for_displaytree!(VotingAndName);


/// A voting function, this is the root for a parsed voting!
#[derive(Debug, Clone)]
pub enum VotingFunction {
    Single(VotingOperation, bool),
    Multi(Vec<VotingOperation>)
}


impl LimitableVotingMethodMarker for VotingFunction {}

impl VotingMethodMarker for VotingFunction {}

impl VotingMethod for VotingFunction {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value>
        where
            A : VotingMethodContext,
            B : VotingMethodContext
    {
        match self {
            VotingFunction::Single(value, _) => {
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

impl DisplayTree for VotingFunction {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingFunction::Single(value, was_root) => {
                if *was_root {
                    write!(f, "{{")?;
                    f.indent(2);
                    write!(f, "\n")?;
                }
                DisplayTree::fmt(value, f)?;
                if *was_root {
                    f.dedent(2);
                    write!(f, "\n")?;
                    write!(f, "}}")?;
                }
                Ok(())
            }
            VotingFunction::Multi(value) => {
                for op in value {
                    DisplayTree::fmt(op, f)?;
                    write!(f, "\n")?;
                }
                Ok(())
            }
        }
    }
}

impl_display_for_displaytree!(VotingFunction);


/// The operation beeing executed
#[derive(Debug, Clone)]
pub enum VotingOperation {
    /// foreach: { <expr> \n <expr> } || for each <expr>
    IterScope {
        expr: VotingExecutableList
    },
    /// global: { <expr> \n <expr> } || <expr>
    GlobalScope {
        expr: VotingExecutableList
    },
    /// aggregate(let <variable_name> = <aggregation>): { <expr> \n <expr> } || <variable_name> = <aggregation> <expr>
    AggregationScope {
        variable_name: String,
        op: Aggregation,
        expr: VotingExecutableList
    },
    /// execute(let <variable_name> = <buildin> || <name_of_registered> || <buildin>(<limit>) || <name_of_registered>(<limit>))
    Execute {
        variable_name: String,
        execution: VotingExecution
    }
}

impl VotingMethod for VotingOperation {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value>
        where
            A : VotingMethodContext,
            B : VotingMethodContext
    {
        match self {
            VotingOperation::IterScope {
                expr
            } => {
                for value in voters {
                    expr.execute(&mut value.combine_with_mut(global_context))?;
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
                            .execute(&mut value.combine_with_mut(global_context))
                            .and_then(|value| value.as_number().map_err(|err| err.into())))
                    .collect::<Result<Vec<_>, _>>()?;
                let new_result = op.calculate_desc(value.into_iter())?;
                global_context.set_value(variable_name.clone(), new_result.into())?;
                Ok(new_result.into())
            }

            VotingOperation::GlobalScope { expr } => {
                Ok(expr.execute(global_context)?)
            }
            VotingOperation::Execute { variable_name, execution } => {
                let result = execution.execute(global_context, voters)?;
                global_context.set_value(variable_name.clone(), result.clone())?;
                Ok(result)
            }
        }
    }
}

impl DisplayTree for VotingOperation {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingOperation::IterScope { expr } => {
                write!(f, "foreach:")?;
                DisplayTree::fmt(expr, f)
            }
            VotingOperation::GlobalScope { expr } => {
                write!(f, "global:")?;
                DisplayTree::fmt(expr, f)
            }
            VotingOperation::AggregationScope { variable_name, op, expr } => {
                write!(f, "aggregate(let {} = {}):", variable_name, op)?;
                DisplayTree::fmt(expr, f)
            }
            VotingOperation::Execute { variable_name, execution } => {
                write!(f, "execute(let {} = ", variable_name)?;
                DisplayTree::fmt(execution, f)?;
                write!(f, ");\n")
            }
        }
    }
}


impl_display_for_displaytree!(VotingOperation);

/// What kind of voting is executed?
#[derive(Debug, Clone)]
pub enum VotingExecution {
    BuildIn(BuildInVoting),
    Parsed(String, Arc<VotingFunction>),
    Limited(VotingWithLimit<Box<VotingExecution>>)
}

impl VotingMethodMarker for VotingExecution {}

impl VotingMethod for VotingExecution {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: VotingMethodContext, B: VotingMethodContext {
        match self {
            VotingExecution::BuildIn(value) => {
                value.execute(global_context, voters)
            }
            VotingExecution::Parsed(_, value) => {
                value.execute(global_context, voters)
            }
            VotingExecution::Limited(value) => {
                value.execute(global_context, voters)
            }
        }
    }
}

impl DisplayTree for VotingExecution {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingExecution::BuildIn(value) => {
                write!(f, "{value}")
            }
            VotingExecution::Parsed(value, _) => {
                write!(f, "{value}")
            }
            VotingExecution::Limited(value) => {
                DisplayTree::fmt(value, f)
            }
        }
    }
}

impl_display_for_displaytree!(VotingExecution);


/// A list of [VotingExpressionOrStatement] elements. Can be a single or multiple.
#[derive(Debug, Clone)]
pub enum VotingExecutableList {
    Single(Box<VotingExpressionOrStatement>),
    Multiple(Vec<VotingExpressionOrStatement>)
}

impl VotingExecutableList {
    pub fn pack_single(expr: VotingExpressionOrStatement) -> Self {
        Self::Single(expr.into())
    }

    pub fn pack_vec(mut values: Vec<VotingExpressionOrStatement>) -> Option<Self> {
        match values.len() {
            0 => None,
            1 => Some(Self::Single(Box::new(values.swap_remove(0)))),
            _ => Some(Self::Multiple(values))
        }
    }
}

impl VotingExecutable for VotingExecutableList {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VotingResult<Value> {
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

impl DisplayTree for VotingExecutableList {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingExecutableList::Single(value) => {
                DisplayTree::fmt(value.as_ref(), f)
            }
            VotingExecutableList::Multiple(value) => {
                write!(f, "{{")?;
                f.indent(2);
                write!(f, "\n")?;
                for (p, v) in value.iter().with_position() {
                    DisplayTree::fmt(v, f)?;
                    match p {
                        Position::First | Position::Middle => {
                            write!(f, "\n")?;
                        }
                        Position::Last | Position::Only => {
                        }
                    }
                }
                f.dedent(2);
                write!(f, "\n")?;
                write!(f, "}}")
            }
        }
    }
}


impl_display_for_displaytree!(VotingExecutableList);


/// An if else expression or statement.
#[derive(Debug, Clone)]
pub struct InnerIfElse {
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

    pub fn from_expr(((cond, if_block), else_block): ((VotingExpression, VotingExecutableList), VotingExecutableList)) -> Self {
        Self::new(cond, if_block, else_block)
    }
}

impl VotingExecutable for InnerIfElse {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VotingResult<Value> {
        if self.cond.execute(context)?.as_boolean()? {
            self.if_block.execute(context)
        } else {
            self.else_block.execute(context)
        }
    }
}

impl DisplayTree for InnerIfElse {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        write!(f, "if(")?;
        DisplayTree::fmt(self.cond.as_ref(), f)?;
        write!(f, ")")?;
        DisplayTree::fmt(&self.if_block, f)?;
        write!(f, " else ")?;
        DisplayTree::fmt(&self.else_block, f)
    }
}


impl_display_for_displaytree!(InnerIfElse);


/// Either a statement or a expression
#[derive(Debug, Clone)]
pub enum VotingExpressionOrStatement {
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
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VotingResult<Value>
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

impl DisplayTree for VotingExpressionOrStatement {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingExpressionOrStatement::Expression { expr } => {
                DisplayTree::fmt(expr, f)
            }
            VotingExpressionOrStatement::Statement { stmt } => {
                DisplayTree::fmt(stmt.as_ref(), f)
            }
        }
    }
}

impl_display_for_displaytree!(VotingExpressionOrStatement);

impl From<VotingExpression> for VotingExpressionOrStatement {
    #[inline]
    fn from(expr: VotingExpression) -> Self {
        Self::Expression {
            expr
        }
    }
}

impl From<VotingStatement> for VotingExpressionOrStatement {
    #[inline]
    fn from(stmt: VotingStatement) -> Self {
        Self::Statement {
            stmt: stmt.into()
        }
    }
}


/// The statements that can be used inside votings
#[derive(Debug, Clone)]
pub enum VotingStatement {
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
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VotingResult<Value>
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

impl DisplayTree for VotingStatement {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingStatement::If { cond, if_block } => {
                write!(f, "if (")?;
                DisplayTree::fmt(cond, f)?;
                write!(f, ")")?;
                DisplayTree::fmt(if_block, f)
            }
            VotingStatement::SetVariable { variable_name, expression } => {
                write!(f, "let {variable_name} = ")?;
                DisplayTree::fmt(expression, f)
            }
        }
    }
}

impl_display_for_displaytree!(VotingStatement);


/// An expression or multiple expressions
#[derive(Clone)]
pub enum VotingExpression {
    Expr(Node),
    IfElse(InnerIfElse),
    TupleGet {
        variable_name: String,
        idx: IndexOrRange
    }
}

impl VotingExpression {
    #[inline(always)]
    pub(crate) fn parse_as_single(s: ParserInput) -> EvalexprResult<Self> {
        Ok(VotingExpression::Expr(evalexpr::build_operator_tree(s.deref())?))
    }
}

impl VotingExecutable for VotingExpression {
    #[inline(always)]
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VotingResult<Value>
    {

        match self {
            VotingExpression::Expr(value) => {
                Ok(value.eval_with_context_mut(context)?)
            }
            VotingExpression::IfElse(value) => {
                value.execute(context)
            }
            VotingExpression::TupleGet { idx, variable_name } => {
                let tuple = context.get_value(variable_name.as_str()).ok_or_else(
                    || EvalexprError::VariableIdentifierNotFound(variable_name.clone())
                )?;
                match tuple {
                    Value::Tuple(value) => {
                        idx.access_value(value).ok_or_else(
                            || VotingExpressionError::TupleGet(
                                variable_name.clone(),
                                idx.clone(),
                                value.len()
                            )
                        )
                    }
                    _ => Err(EvalexprError::expected_tuple(tuple.clone()).into())
                }
            }
        }
    }
}

impl DisplayTree for VotingExpression {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingExpression::Expr(value) => {
                write!(f, "{}", walk_left_to_right(value))
            }
            VotingExpression::IfElse(value) => {
                DisplayTree::fmt(value, f)
            }
            VotingExpression::TupleGet { idx, variable_name } => {
                write!(f, "{variable_name}[{idx}]")
            }
        }
    }
}

impl_display_for_displaytree!(VotingExpression);

impl Debug for VotingExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VotingExpression::Expr(value) => {f.write_str(&value.to_string())}
            VotingExpression::IfElse(value) => {
                f.debug_struct("IfElse").field("if_else", value).finish()
            }
            VotingExpression::TupleGet {idx, variable_name} => {
                f.debug_struct("TupleGet")
                    .field("idx", idx)
                    .field("var_name", variable_name)
                    .finish()
            }
        }
    }
}

/// A parsed index or range.
#[derive(Debug, Clone)]
pub enum IndexOrRange {
    Index(usize),
    Range(Range<usize>),
    RangeTo(RangeTo<usize>),
    RangeFrom(RangeFrom<usize>),
    RangeInclusive(RangeInclusive<usize>),
    RangeToInclusive(RangeToInclusive<usize>),
    RangeFull
}

impl IndexOrRange {
    pub fn access_value(&self, target: &TupleType) -> Option<Value> {
        match self {
            IndexOrRange::Index(value) => {
                target.get(*value).cloned()
            }
            IndexOrRange::Range(value) => {
                target.get(value.clone()).map(
                    |value| Value::Tuple(value.iter().cloned().collect())
                )
            }
            IndexOrRange::RangeTo(value) => {
                target.get(value.clone()).map(
                    |value| Value::Tuple(value.iter().cloned().collect())
                )
            }
            IndexOrRange::RangeFrom(value) => {
                target.get(value.clone()).map(
                    |value| Value::Tuple(value.iter().cloned().collect())
                )
            }
            IndexOrRange::RangeInclusive(value) => {
                target.get(value.clone()).map(
                    |value| Value::Tuple(value.iter().cloned().collect())
                )
            }
            IndexOrRange::RangeToInclusive(value) => {
                target.get(value.clone()).map(
                    |value| Value::Tuple(value.iter().cloned().collect())
                )
            }
            IndexOrRange::RangeFull => {
                target.get(..).map(
                    |value| Value::Tuple(value.iter().cloned().collect())
                )
            }
        }
    }
}

impl Display for IndexOrRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexOrRange::Index(value) => {
                write!(f, "{value}")
            }
            IndexOrRange::RangeTo(value) => {
                write!(f, "..{}", value.end)
            }
            IndexOrRange::Range(value) => {
                write!(f, "{}..{}", value.start, value.end)
            }
            IndexOrRange::RangeFull => {
                write!(f, "..")
            }
            IndexOrRange::RangeFrom(value) => {
                write!(f, "{}..", value.start)
            }
            IndexOrRange::RangeInclusive(value) => {
                write!(f, "{}..={}", value.start(), value.end())
            }
            IndexOrRange::RangeToInclusive(value) => {
                write!(f, "..={}", value.end)
            }
        }
    }
}


