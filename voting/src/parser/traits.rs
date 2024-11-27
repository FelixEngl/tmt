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

use evalexpr::{ContextWithMutableVariables, Value};
use crate::constants::TMTNumericTypes;
use crate::display::DisplayTree;
use crate::VotingResult;

/// Marks a struct as voting executable. An executable does only know a single context that can be modified.
pub(crate) trait VotingExecutable: DisplayTree {
    fn execute(&self, context: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VotingResult<Value<TMTNumericTypes>>;
}
