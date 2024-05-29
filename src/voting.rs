use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::{Deref, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice::SliceIndex;
use std::str::FromStr;
use evalexpr::{Context, ContextWithMutableVariables, EvalexprError, EvalexprResult, Node, Operator, TupleType, Value};
use itertools::{FoldWhile, Itertools, Position};
use strum::{EnumIs};
use thiserror::Error;
use crate::toolkit::evalexpr::CombineableContext;
use crate::voting::aggregations::{Aggregation, AggregationError};

mod parser;
mod aggregations;


#[derive(Debug, Error)]
pub enum VotingExpressionError {
    #[error(transparent)]
    Eval(#[from] EvalexprError),
    #[error(transparent)]
    Agg(#[from] AggregationError),
    #[error("The tuple {0} with length {2} does not have a value at {1}!")]
    TupleGet(String, IndexOrRange, usize),
}


type VResult<T> = Result<T, VotingExpressionError>;




pub trait VotingMethod {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VResult<Value>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables;
}


pub struct Voting<T: VotingMethod> {
    name: String,
    expr: T
}

impl<T> VotingMethod for Voting<T> where T: VotingMethod {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        self.expr.execute(global_context, voters)
    }
}



trait DisplayTree: Display {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result;
}



macro_rules! impl_display_for_displaytree {
    ($($target: ident),+) => {
        $(
            impl Display for $target {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    let mut code_formatter = IndentWriter::new(f);
                    DisplayTree::fmt(self, &mut code_formatter)
                }
            }
        )+
    };
}


#[derive(Debug)]
enum VotingFunction {
    Single(VotingOperation),
    Multi(Vec<VotingOperation>)
}

impl VotingMethod for VotingFunction {
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

struct IndentWriter<'a, T: Write> {
    f: &'a mut T,
    level: usize,
    indent: String
}

impl<'a, T> IndentWriter<'a, T> where T: Write {
    fn new(f: &'a mut T) -> Self {
        Self {
            f,
            level: 0,
            indent: String::new()
        }
    }

    fn indent(&mut self, value: usize) {
        self.level = self.level.saturating_add(value);
        self.indent = " ".repeat(self.level);
    }
    
    fn dedent(&mut self, value: usize) {
        self.level = self.level.saturating_sub(value);
        self.indent = " ".repeat(self.level);
    }
}

impl<T> Write for IndentWriter<'_, T> where T: Write {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if s.ends_with("\n") {
            write!(self.f, "{}{}", s, self.indent)
        } else {
            write!(self.f, "{}", s)
        }

    }
}



impl DisplayTree for VotingFunction {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        match self {
            VotingFunction::Single(value) => {
                DisplayTree::fmt(value, f)
            }
            VotingFunction::Multi(value) => {
                for op in value {
                    DisplayTree::fmt(op, f)?;
                }
                Ok(())
            }
        }
    }
}

impl_display_for_displaytree!(VotingFunction);



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

impl VotingMethod for VotingOperation {
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

impl DisplayTree for VotingOperation {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        let expr = match self {
            VotingOperation::IterScope { expr } => {
                write!(f, "foreach:")?;
                expr
            }
            VotingOperation::GlobalScope { expr } => {
                write!(f, "global:")?;
                expr
            }
            VotingOperation::AggregationScope { variable_name, op, expr } => {
                write!(f, "aggregate({} = {}):", variable_name, op)?;
                expr
            }
        };
        DisplayTree::fmt(expr, f)?;
        Ok(())
    }
}

impl_display_for_displaytree!(VotingOperation);

trait VotingExecutable: DisplayTree {
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
#[derive(Debug)]
enum VotingExpressionOrStatement {
    Expression {
        expr: VotingExpression
    },
    Statement {
        stmt: Box<VotingStatement>
    }
}

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
enum VotingExpression {
    Expr(Node),
    IfElse(InnerIfElse),
    TupleGet {
        variable_name: String,
        idx: IndexOrRange
    }
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


#[derive(Debug, Clone)]
enum IndexOrRange {
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


#[derive(Debug, Clone)]
#[derive(EnumIs)]
enum NodeContainer<'a> {
    Leaf(&'a Node, bool),
    Single(&'a Node, Box<NodeContainer<'a>>, bool),
    Expr(Box<NodeContainer<'a>>, &'a Node, Box<NodeContainer<'a>>, bool),
    Special(&'a Node, Vec<NodeContainer<'a>>, bool)
}

fn walk_left_to_right(node: &Node) -> NodeContainer {
    fn walk_left_to_right_(node: &Node, is_root: bool) -> NodeContainer {
        let children = node.children();
        match children.len() {
            0 => {
                NodeContainer::Leaf(node, is_root)
            }
            1 => {
                NodeContainer::Single(node, walk_left_to_right_(&children[0], false).into(), is_root)
            }
            2 => {
                NodeContainer::Expr(
                    walk_left_to_right_(&children[0], false).into(),
                    node,
                    walk_left_to_right_(&children[1], false).into(),
                    is_root
                )
            }
            _ => {
                NodeContainer::Special(node, node.children().iter().map(
                    |value| walk_left_to_right_(value, false)
                ).collect_vec(), is_root)
            }
        }
    }
    walk_left_to_right_(node, true)
}

impl<'a> NodeContainer<'a> {

    fn origin(&self) -> &'a Node {
        match self {
            NodeContainer::Leaf(value, _) => {*value}
            NodeContainer::Single(value, _, _) => {*value}
            NodeContainer::Expr(_, value, _, _) => {*value}
            NodeContainer::Special(value, _, _) => {*value}
        }
    }
}

impl Display for NodeContainer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeContainer::Leaf(value, _) => {
                write!(f, "{}", format!("{}", value).trim())
            }
            NodeContainer::Single(value1, value2, is_root) => {
                if *is_root {
                    write!(f, "{}", value2.as_ref())
                } else {
                    match value1.operator() {
                        Operator::RootNode => {
                            if value2.is_expr() {
                                write!(f, "({})", value2.as_ref())
                            } else {
                                write!(f, "{}", value2.as_ref())
                            }
                        }
                        _ => {
                            write!(f, "{}{}", format!("{}", value1.operator()).trim(), value2.as_ref())
                        }
                    }
                }
            }
            NodeContainer::Expr(value1, value2, value3, _) => {
                write!(f, "{} {} {}", value1, format!("{}", value2.operator()).trim(), value3)
            }
            NodeContainer::Special(value1, value2, _) => {
                match value1.operator() {
                    Operator::Tuple => {
                        write!(f, "({})", value2.iter().join(", "))
                    }
                    Operator::Chain => {
                        write!(f, "{}", value2.iter().join("; "))
                    }
                    _ => write!(f, "[!{value1}]")
                }

            }
        }
    }
}

pub(crate) mod parse {
    use evalexpr::EvalexprError;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{alphanumeric1, char, digit0, digit1, multispace0, multispace1, one_of, space0, space1};
    use nom::combinator::{cut, map, map_res, not, opt, peek, recognize};
    use nom::error::{context, ContextError, FromExternalError, ParseError};
    use nom::{AsChar, InputIter, InputTakeAtPosition, IResult, Parser};
    use nom::multi::{many1, many1_count};
    use nom::sequence::{delimited, preceded, terminated, tuple};
    use strum::{AsRefStr, Display, EnumString};
    use thiserror::Error;
    use crate::voting::{IndexOrRange, InnerIfElse, VotingExecutableList, VotingExpression, VotingExpressionOrStatement, VotingFunction, VotingOperation, VotingStatement};
    use crate::voting::aggregations::parse::AggregationParserError;
    use crate::voting::parse::VotingParseError::{EmptyIndexNotAllowed, NoVotingFound, ToRangeAlwaysNeedsValue};

    const IMPORTANT_TOKENS: &str = "+-*/%^=!<>&|,;";

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
                        recognize(one_of("._+-*/%^=!<>&|,;: \"'"))
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
                                Err(ToRangeAlwaysNeedsValue)
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
                            Err(EmptyIndexNotAllowed)
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
        use crate::voting::parse::{voting_function};
        use crate::voting::VotingMethod;

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
}


