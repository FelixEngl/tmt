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

// Inspired by https://github.com/billyrieger/bimap-rs/blob/main/src/mem.rs

use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash};
use std::ops::{Bound, Deref};
use std::sync::Arc;

/// A ref that supplies the Hash and Eq method of the underlying struct.
/// It is threadsafe and allows a simple cloning as well as ordering
/// and dereferencing of the underlying value.
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct HashRef<T> {
    inner: Arc<T>
}

unsafe impl<T> Sync for HashRef<T>{}
unsafe impl<T> Send for HashRef<T>{}

impl<T> HashRef<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value)
        }
    }
}

impl<T: Display> Display for HashRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Clone for HashRef<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T> Deref for HashRef<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: Debug> Debug for HashRef<T>  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}


impl<T> From<T> for HashRef<T>  {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}


#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Wrapper<T: ?Sized>(pub T);

impl<T: ?Sized> Wrapper<T> {
    pub fn wrap(value: &T) -> &Self {
        unsafe { &*(value as *const T as *const Self) }
    }

    pub fn wrap_bound(bound: Bound<&T>) -> Bound<&Self> {
        match bound {
            Bound::Included(t) => Bound::Included(Self::wrap(t)),
            Bound::Excluded(t) => Bound::Excluded(Self::wrap(t)),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

impl<K, Q> Borrow<Wrapper<Q>> for HashRef<K>
where
    K: Borrow<Q>,
    Q: ?Sized,
{
    fn borrow(&self) -> &Wrapper<Q> {
        // Rc<K>: Borrow<K>
        let k: &K = self.inner.borrow();
        // K: Borrow<Q>
        let q: &Q = k.borrow();

        Wrapper::wrap(q)
    }
}
