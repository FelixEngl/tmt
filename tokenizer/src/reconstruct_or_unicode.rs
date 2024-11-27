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

use charabia::{ReconstructedTokenIter, Token};
use crate::unicode_segmenter::UnicodeSegmenterTokenIter;

pub enum SegmentedIter<'o, 'tb> {
    Unicode(UnicodeSegmenterTokenIter<'o, 'tb>),
    Reconstructor(ReconstructedTokenIter<'o, 'tb>),
}


impl<'o, 'tb> Iterator for SegmentedIter<'o, 'tb> {
    type Item = (&'o str, Token<'o>);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SegmentedIter::Unicode(ref mut value) => {
                value.next()
            }
            SegmentedIter::Reconstructor(ref mut value) => {
                value.next()
            }
        }
    }
}