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

use std::fmt::Debug;
use std::iter::{FusedIterator};

pub trait SupportsTupling {
    fn tuple_first<V: Clone>(self, value: V) -> TupleFirst<Self, V> where Self: Iterator + Sized;
    fn tuple_last<V: Clone>(self, value: V) -> TupleLast<Self, V> where Self: Iterator + Sized;
}

impl<I: Iterator> SupportsTupling for I {
    fn tuple_first<V: Clone>(self, value: V) -> TupleFirst<Self, V> where Self: Iterator {
        TupleFirst::new(self, value)
    }

    fn tuple_last<V: Clone>(self, value: V) -> TupleLast<Self, V> where Self: Iterator {
        TupleLast::new(self, value)
    }
}

#[derive(Debug)]
pub struct TupleFirst<I, V> {
    iter: I,
    value: V
}

impl<I, V> TupleFirst<I, V>{
    pub(crate) fn new(iter: I, value: V) -> Self {
        Self { iter, value }
    }
}


impl<I: Iterator, V: Clone> Iterator for TupleFirst<I, V> {
    type Item = (V, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|value| (self.value.clone(), value))
    }
}

impl<I: DoubleEndedIterator, V: Clone> DoubleEndedIterator for TupleFirst<I, V>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|value| (self.value.clone(), value))
    }
}

impl<I: ExactSizeIterator, V: Clone> ExactSizeIterator for TupleFirst<I, V>
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: FusedIterator, V: Clone> FusedIterator for TupleFirst<I, V>
{
}



#[derive(Debug)]
pub struct TupleLast<I, V> {
    iter: I,
    value: V
}

impl<I, V> TupleLast<I, V> {
    pub(crate) fn new(iter: I, value: V) -> Self {
        Self { iter, value }
    }
}

impl<I: Iterator, V: Clone> Iterator for TupleLast<I, V> {
    type Item = (I::Item, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|value| (value, self.value.clone()))
    }
}

impl<I: DoubleEndedIterator, V: Clone> DoubleEndedIterator for TupleLast<I, V>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|value| (value, self.value.clone()))
    }
}

impl<I: ExactSizeIterator, V: Clone> ExactSizeIterator for TupleLast<I, V>
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: FusedIterator, V: Clone> FusedIterator for TupleLast<I, V>
{
}

