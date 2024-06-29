use std::borrow::Borrow;
use std::cmp::{max, min};
use std::fmt::{Display};
use std::hash::{Hash};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Offset {
    begin: usize,
    end: usize,
}

impl Offset {
    pub fn new(begin: usize, end: usize) -> Option<Self> {
        if begin >= end {
            None
        } else {
            Some(
                Self {
                    begin,
                    end
                }
            )
        }
    }


    fn combine(&self, other: &Offset) -> Offset {
        Self {
            begin: min(self.begin, other.begin),
            end: max(self.end, other.end)
        }
    }

}