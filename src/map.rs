use crate::{
    bucket::{Bucket, BucketValue, BUCKET_CAP},
    guard::Ref,
    RefMut,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

/// A set backed by Extendable Hashing.
#[derive(Debug)]
pub struct HashMap<K, V> {
    /// amount of elements
    len: usize,
    /// Global depth
    global_depth: usize,
    /// Directory entries
    directories: Vec<Option<Arc<RwLock<Bucket<K, V>>>>>,
}

impl<K, V> HashMap<K, V> {
    /// Create an empty `HashMap`.
    pub fn new() -> Self {
        let bucket0 = Arc::new(RwLock::new(Bucket::new(&[0])));
        let bucket1 = Arc::new(RwLock::new(Bucket::new(&[1])));

        Self {
            len: 0,
            global_depth: 1,
            directories: vec![Some(bucket0), Some(bucket1)],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.directories.len() * BUCKET_CAP
    }
}

impl<K: Hash, V> HashMap<K, V> {
    /// Calculate which bucket `value` will go to.
    fn locate_bucket(&self, key: &K) -> usize {
        let mut default_hasher = DefaultHasher::new();
        key.hash(&mut default_hasher);
        let hash_res = default_hasher.finish();

        // Use the last `self.global` bits as it is easy to calculate
        (hash_res % 2_u64.pow(self.global_depth as _)) as usize
    }

    fn split(&mut self, bucket: ()) {
        unimplemented!()
    }

    /// Insert `value` to this set.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Eq,
    {
        let bucket_idx = self.locate_bucket(&key);
        let bucket = self.directories[bucket_idx]
            .as_ref()
            .expect("Should be Some");
        let mut bucket_guard = bucket.write().unwrap();

        // Check existence
        if bucket_guard.keys.contains(&key) {
            return Some(value);
        }

        if !bucket_guard.is_full() {
            let _ = bucket_guard.keys.push_within_capacity(key);
            let _ = bucket_guard.values.push_within_capacity(value);

            self.len += 1;
        } else {
            todo!("split")
        }

        None
    }

    pub fn get(&self, key: &K) -> Option<Ref<'_, K, V>>
    where
        K: Eq,
    {
        let bucket_idx = self.locate_bucket(key);
        let read_guard = self.directories[bucket_idx]
            .as_ref()
            .unwrap()
            .read()
            .unwrap();

        let idx = read_guard.keys.iter().position(|item| item == key)?;

        Some(Ref::new(read_guard, idx))
    }

    pub fn get_mut(&self, key: &K) -> Option<RefMut<'_, K, V>>
    where
        K: Eq,
    {
        let bucket_idx = self.locate_bucket(key);
        let read_guard = self.directories[bucket_idx]
            .as_ref()
            .unwrap()
            .write()
            .unwrap();

        let idx = read_guard.keys.iter().position(|item| item == key)?;

        Some(RefMut::new(read_guard, idx))
    }
}
