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

use std::sync::Arc;
use evalexpr::{Value};
use nom::branch::alt;
use nom::combinator::{map, map_res};
use nom::IResult;
use strum::EnumIs;
use crate::{BuildInVoting, VotingFunction, VotingMethod, VotingMethodContext, VotingResult, VotingWithLimit};
use crate::constants::TMTNumericTypes;
use crate::parser::input::ParserInput;
use crate::parser::logic::{build_in_voting, ErrorType, global_voting_function, parse_limited, variable_name, voting};
use crate::parser::logic::VotingParseError::{NoRegistryProvided, NoVotingInRegistryFound};
use crate::parser::voting_function::VotingAndName;
use crate::traits::VotingMethodMarker;

pub(crate) mod voting_function;
pub mod logic;
mod traits;
pub mod input;


/// Parse the [ParserInput] to a [InterpretedVoting]
pub fn parse<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, InterpretedVoting, E> {
    fn parse_internal<'a, 'b, E: ErrorType<ParserInput<'a,'b>>>(input: ParserInput<'a,'b>) -> IResult<ParserInput<'a,'b>, InterpretedVoting, E> {
        alt((
            map(build_in_voting, InterpretedVoting::BuildIn),
            map_res(variable_name, |value| match value.registry() {
                None => {Err(NoRegistryProvided)}
                Some(registry) => {
                    registry
                        .get(value.as_ref())
                        .ok_or_else(|| NoVotingInRegistryFound(value.to_string()))
                        .map(InterpretedVoting::FromRegistry)
                }
            }),
            map(voting, InterpretedVoting::ForRegistry),
            map(global_voting_function, InterpretedVoting::Parsed),
        ))(input)
    }

    alt((
        map(parse_limited(parse_internal), InterpretedVoting::Limited),
        parse_internal
    ))(input)
}

/// What kind of voting did we parse?
#[derive(Debug, EnumIs, Clone)]
pub enum InterpretedVoting {
    BuildIn(BuildInVoting),
    FromRegistry(Arc<VotingFunction>),
    Parsed(VotingFunction),
    ForRegistry(VotingAndName),
    Limited(VotingWithLimit<Box<InterpretedVoting>>),
}

impl VotingMethodMarker for InterpretedVoting {}

impl VotingMethod for InterpretedVoting {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A : VotingMethodContext,
        B : VotingMethodContext {
        match self {
            InterpretedVoting::BuildIn(value) => {
                value.execute(global_context, voters)
            }
            InterpretedVoting::FromRegistry(value) => {
                value.execute(global_context, voters)
            }
            InterpretedVoting::Parsed(value) => {
                value.execute(global_context, voters)
            }
            InterpretedVoting::ForRegistry(value) => {
                value.1.execute(global_context, voters)
            }
            InterpretedVoting::Limited(value) => {
                value.execute(global_context, voters)
            }
        }
    }
}

impl From<Arc<VotingFunction>> for InterpretedVoting {
    fn from(value: Arc<VotingFunction>) -> Self {
        Self::FromRegistry(value)
    }
}

impl From<VotingFunction> for InterpretedVoting {
    fn from(value: VotingFunction) -> Self {
        Self::Parsed(value)
    }
}

impl From<VotingAndName> for InterpretedVoting {
    fn from(value: VotingAndName) -> Self {
        Self::ForRegistry(value)
    }
}

impl From<BuildInVoting> for InterpretedVoting {
    fn from(value: BuildInVoting) -> Self {
        Self::BuildIn(value)
    }
}


#[cfg(test)]
mod test {
    use nom::{Finish, IResult};
    use crate::BuildInVoting;
    use crate::parser::input::ParserInput;
    use crate::parser::logic::global_voting_function;
    use crate::parser::{parse, InterpretedVoting};
    use crate::registry::VotingRegistry;
    use crate::voting::BuildInVoting;
    use crate::voting::parser::{parse, InterpretedVoting};
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
        if let InterpretedVoting::Limited(inner) = result {
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
        if let InterpretedVoting::Limited(inner) = result {
            assert!(inner.expr.is_parsed())
        }
    }
}