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


use evalexpr::ContextWithMutableVariables;
use strum::{Display, EnumIs};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[derive(Display, EnumIs)]
pub enum ExtensionLevel {
    Global,
    Topic,
    Voter
}

impl ExtensionLevel {
    pub const fn is_at_least_on(&self, other: ExtensionLevel) -> bool {
        match self {
            ExtensionLevel::Global => true,
            ExtensionLevel::Topic => !other.is_voter(),
            ExtensionLevel::Voter => other.is_voter()
        }
    }
}

/// A trait that marks an element as an extender for a topic.
pub trait ContextExtender {
    /// Mars on which level it is to be applied
    const EXTENSION_LEVEL: ExtensionLevel;

    /// Allows to add model specific variables.
    /// Panics when invalid variables are added.
    fn extend_context(&self, context: &mut impl ContextWithMutableVariables);
}
