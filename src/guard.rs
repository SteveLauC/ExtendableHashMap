use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::bucket::Bucket;

#[derive(Debug)]
pub struct Ref<'a, K, V> {
    read_guard: RwLockReadGuard<'a, Bucket<K, V>>,
    idx: usize,
}

impl<'a, K, V> Ref<'a, K, V> {
    pub(crate) fn new(
        guard: RwLockReadGuard<'a, Bucket<K, V>>,
        idx: usize,
    ) -> Self {
        Self {
            read_guard: guard,
            idx,
        }
    }
    pub fn key(&self) -> &K {
        &self.read_guard.keys[self.idx]
    }

    pub fn value(&self) -> &V {
        &self.read_guard.values[self.idx]
    }
}

#[derive(Debug)]
pub struct RefMut<'a, K, V> {
    write_guard: RwLockWriteGuard<'a, Bucket<K, V>>,
    idx: usize
}


impl<'a, K, V> RefMut<'a, K, V> {
    pub(crate) fn new(
        guard: RwLockWriteGuard<'a, Bucket<K, V>>,
        idx: usize,
    ) -> Self {
        Self {
            write_guard: guard,
            idx,
        }
    }
    pub fn key(&self) -> &K {
        &self.write_guard.keys[self.idx]
    }

    pub fn value(&self) -> &V {
        &self.write_guard.values[self.idx]
    }
}
