use std::ops::Deref;
use std::ops::DerefMut;

use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

pub enum LockWrapper<'a, T> {
    Read(RwLockReadGuard<'a, T>),
    Write(RwLockWriteGuard<'a, T>),
}

impl<'a, T> Deref for LockWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            LockWrapper::Read(read_guard) => read_guard.deref(),
            LockWrapper::Write(write_guard) => write_guard.deref(),
        }
    }
}

impl<'a, T> DerefMut for LockWrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            LockWrapper::Write(write_gaurd) => write_gaurd.deref_mut(),
            LockWrapper::Read(_) => panic!(),
        }
    }
}
