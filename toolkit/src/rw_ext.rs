use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use sealed::sealed;

#[sealed]
pub trait RWLockUnwrapped<T: ?Sized> {
    fn read_unwrapped<'a>(&'a self) -> RwLockReadGuard<'a, T> where T: 'a;
    fn write_unwrapped<'a>(&'a self) -> RwLockWriteGuard<'a, T> where T: 'a;
}

#[sealed]
impl<T: ?Sized> RWLockUnwrapped<T> for RwLock<T> {
    fn read_unwrapped<'a>(&'a self) -> RwLockReadGuard<'a, T>
    where
        T: 'a
    {
        self.read().unwrap()
    }

    fn write_unwrapped<'a>(&'a self) -> RwLockWriteGuard<'a, T>
    where
        T: 'a
    {
        self.write().unwrap()
    }
}