use charabia::{ReconstructedTokenIter, Token};
use crate::tokenizer::unicode_segmenter::UnicodeSegmenterTokenIter;

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