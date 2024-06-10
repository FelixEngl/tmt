use std::sync::Arc;
use evalexpr::{ContextWithMutableVariables, Value};
use nom::branch::alt;
use nom::combinator::{map, map_res};
use nom::IResult;
use strum::EnumIs;
use crate::voting::{BuildInVoting, VotingFunction, VotingMethod, VotingResult, VotingWithLimit};
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::logic::{build_in_voting, ErrorType, global_voting_function, parse_limited, variable_name, voting};
use crate::voting::parser::logic::VotingParseError::{NoRegistryProvided, NoVotingInRegistryFound};
use crate::voting::parser::voting_function::VotingAndName;
use crate::voting::traits::VotingMethodMarker;

pub(crate) mod voting_function;
pub mod logic;
mod traits;
pub mod input;

#[derive(Debug, EnumIs, Clone)]
pub enum ParseResult {
    BuildIn(BuildInVoting),
    FromRegistry(Arc<VotingFunction>),
    Parsed(VotingFunction),
    ForRegistry(VotingAndName),
    Limited(VotingWithLimit<Box<ParseResult>>)
}

pub fn parse<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, ParseResult, E> {
    fn parse_internal<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, ParseResult, E> {
        alt((
            map(build_in_voting, ParseResult::BuildIn),
            map_res(variable_name, |value| match value.registry() {
                None => {Err(NoRegistryProvided)}
                Some(registry) => {
                    registry
                        .get(value.as_ref())
                        .ok_or_else(|| NoVotingInRegistryFound(value.to_string()))
                        .map(ParseResult::FromRegistry)
                }
            }),
            map(voting, ParseResult::ForRegistry),
            map(global_voting_function, ParseResult::Parsed),
        ))(input)
    }

    alt((
        map(parse_limited(parse_internal), ParseResult::Limited),
        parse_internal
    ))(input)
}

impl VotingMethodMarker for ParseResult{}

impl VotingMethod for ParseResult {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        match self {
            ParseResult::BuildIn(value) => {
                value.execute(global_context, voters)
            }
            ParseResult::FromRegistry(value) => {
                value.execute(global_context, voters)
            }
            ParseResult::Parsed(value) => {
                value.execute(global_context, voters)
            }
            ParseResult::ForRegistry(value) => {
                value.1.execute(global_context, voters)
            }
            ParseResult::Limited(value) => {
                value.execute(global_context, voters)
            }
        }
    }
}


impl From<Arc<VotingFunction>> for ParseResult {
    fn from(value: Arc<VotingFunction>) -> Self {
        Self::FromRegistry(value)
    }
}

impl From<VotingFunction> for ParseResult {
    fn from(value: VotingFunction) -> Self {
        Self::Parsed(value)
    }
}

impl From<VotingAndName> for ParseResult {
    fn from(value: VotingAndName) -> Self {
        Self::ForRegistry(value)
    }
}

impl From<BuildInVoting> for ParseResult {
    fn from(value: BuildInVoting) -> Self {
        Self::BuildIn(value)
    }
}


#[cfg(test)]
mod test {
    use nom::{Finish, IResult};
    use crate::voting::BuildInVoting;
    use crate::voting::parser::{parse, ParseResult};
    use crate::voting::parser::input::ParserInput;
    use crate::voting::parser::logic::global_voting_function;
    use crate::voting::registry::VotingRegistry;

    #[test]
    fn can_recognize_buildin(){
        let build_ind = BuildInVoting::CombSumPow2RRPow2.to_string();
        let result: IResult<_, _> = parse(build_ind.as_str().into());
        let (_, result) = result.unwrap();
        assert!(result.is_build_in())
    }

    #[test]
    fn can_recognize_parsed(){
        let result: IResult<_, _> = parse("aggregate(let sss = sumOf): score".into());
        let (_, result) = result.unwrap();
        assert!(result.is_parsed())
    }

    #[test]
    fn can_recognize_from_registry(){

        let result = global_voting_function::<nom::error::Error<_>>("
            aggregate(let sss = avgOf): { score + 1 }
            global: sss
        ".into()).unwrap().1;

        let registry = VotingRegistry::new();
        registry.register("call_me".to_string(), result);



        let result: IResult<_, _> = parse(ParserInput::new(
            "call_me",
            &registry
        ));
        let (_, result) = result.unwrap();
        assert!(result.is_from_registry())
    }

    #[test]
    fn can_recognize_parsed_multiline(){
        let result: Result<_, _> = parse::<nom::error::VerboseError<_>>("{
            aggregate(let sss = sumOf): {score}
            global: sss
        }".into()).finish();

        let (_, result) = result.unwrap();
        assert!(result.is_parsed())
    }

    #[test]
    fn can_recognize_parsed_for_registry(){
        let result: IResult<_, _> = parse("declare my_vote {
            aggregate(let sss = sumOf): score
            global: sss
        }".into());
        let (_, result) = result.unwrap();
        assert!(result.is_for_registry())
    }

    #[test]
    fn can_recognize_limited(){
        let result: IResult<_, _> = parse("Voters(20)".into());
        let (_, result) = result.unwrap();
        assert!(result.is_limited());
        if let ParseResult::Limited(inner) = result {
            assert!(inner.expr.is_build_in())
        }
    }

    #[test]
    fn can_recognize_limited_multiline(){
        let result: IResult<_, _> = parse("{
            aggregate(let sss = sumOf): {score}
            global: sss
        }(20)".into());
        let (_, result) = result.unwrap();
        assert!(result.is_limited());
        if let ParseResult::Limited(inner) = result {
            assert!(inner.expr.is_parsed())
        }
    }
}