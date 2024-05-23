// Taken from https://github.com/billyrieger/bimap-rs/blob/main/src/mem.rs

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// A ref that supplies the Hash and Eq method of the underlying struct.
/// It is threadsafe and allows a simple cloning as well as ordering
/// and dereferencing of the underlying value.
#[derive(Debug)]
#[repr(transparent)]
pub struct HashRef<T: ?Sized> {
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

impl<T: Hash> Hash for HashRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T: Display> Display for HashRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: ?Sized + PartialEq> PartialEq for HashRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}
impl<T: ?Sized + Eq> Eq for HashRef<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for HashRef<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T: ?Sized + Ord> Ord for HashRef<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> Clone for HashRef<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T: ? Sized> Deref for HashRef<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

// impl<T: ?Sized> Borrow<T> for HashRef<T> where T: Eq + Hash {
//     #[inline]
//     fn borrow(&self) -> &T {
//         self.inner.borrow()
//     }
// }

// impl<Q: ?Sized, T: ?Sized> Borrow<Q> for HashRef<T> where Q: Eq + Hash, T: Borrow<Q> {
//     #[inline]
//     fn borrow(&self) -> &Q {
//         let v: &T = self.inner.borrow();
//         v.borrow()
//     }
// }

impl<T: ?Sized> AsRef<T> for HashRef<T>  {
    fn as_ref(&self) -> &T {
        self.inner.as_ref()
    }
}


impl<T> From<T> for HashRef<T>  {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
