use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Debug)]
pub enum OwnedOrArcRw<T> {
    Owned(T),
    Arc(Arc<RwLock<T>>)
}

impl<T> OwnedOrArcRw<T> {
    pub fn get<'a>(&'a self) -> OwnedOrArcRwRef<'a, T> {
        match self {
            OwnedOrArcRw::Owned(value) => {
                OwnedOrArcRwRef::Owned(value)
            }
            OwnedOrArcRw::Arc(value) => {
                OwnedOrArcRwRef::Arc(value.read().unwrap())
            }
        }
    }

    pub fn to_arc(self) -> Arc<RwLock<T>> {
        match self {
            OwnedOrArcRw::Owned(value) => {
                Arc::new(RwLock::new(value))
            }
            OwnedOrArcRw::Arc(value) => {
                value
            }
        }
    }
}


pub enum OwnedOrArcRwRef<'a, T> {
    Owned(&'a T),
    Arc(RwLockReadGuard<'a, T>)
}

impl<T> Deref for OwnedOrArcRwRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            OwnedOrArcRwRef::Owned(value) => {*value}
            OwnedOrArcRwRef::Arc(value) => {
                value.deref()
            }
        }
    }
}

impl<T> From<T> for OwnedOrArcRw<T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<T> From<Arc<RwLock<T>>> for OwnedOrArcRw<T> {
    fn from(value: Arc<RwLock<T>>) -> Self {
        Self::Arc(value)
    }
}
