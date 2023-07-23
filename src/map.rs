use crate::bucket::{Bucket, BucketValue, BUCKET_CAP};
use std::{
    borrow::Borrow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// A set backed by Extendable Hashing.
#[derive(Debug)]
pub struct HashMap<K, V> {
    /// The number of elements
    len: usize,
    /// Global depth
    global_depth: usize,
    /// Directory entries, storing the index of its
    /// corresponding bucket.
    directories: Vec<usize>,
    /// Buckets
    buckets: Vec<Bucket<K, V>>,
}

impl<K, V> HashMap<K, V> {
    /// Return the global depth of this HashMap.
    #[inline]
    fn global_depth(&self) -> usize {
        self.global_depth
    }

    /// Create an empty `HashMap`.
    pub fn new() -> Self {
        let bucket0 = Bucket::new(&[0]);
        let bucket1 = Bucket::new(&[1]);

        Self {
            len: 0,
            global_depth: 1,
            directories: vec![0, 1],
            buckets: vec![bucket0, bucket1],
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the number of elements the map can hold
    /// without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.directories.len() * BUCKET_CAP
    }
}

impl<K: Hash, V> HashMap<K, V> {
    /// Calculate which bucket `value` will go to.
    fn locate_bucket<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash,
    {
        let mut default_hasher = DefaultHasher::new();
        key.hash(&mut default_hasher);
        let hash_res = default_hasher.finish();

        // Use the last `self.global` bits as it is easy to
        // calculate
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
        let bucket = self
            .buckets
            .get_mut(bucket_idx)
            .expect("locate_bucket() returns a wrong index");

        // Check existence
        if bucket.keys.contains(&key) {
            return Some(value);
        }

        if !bucket.is_full() {
            // ignore the returned Result as calling
            // `unwrap()` on it requires the
            // Debug impl of `V`
            let _ = bucket.keys.push_within_capacity(key);
            let _ = bucket.values.push_within_capacity(value);

            self.len += 1;
        } else {
            let local_depth = bucket.local_depth();
            let global_depth = self.global_depth;
            assert!(local_depth <= global_depth);

            if local_depth < global_depth {
                let last_half_range = bucket
                    .value(global_depth)
                    .last_half_range()
                    .expect("Should be range");
                let mut bucket_slice = bucket.bits.clone();
                bucket.bits.push(0);
                bucket_slice.push(1);
                let new_bucket = Bucket::new(bucket_slice.as_slice());
                let new_bucket_idx = self.buckets.len();
                self.buckets.push(new_bucket);

                // redistribute pointers
                for idx in last_half_range {
                    self.directories[idx] = new_bucket_idx;
                }
            } else {
            }
        }

        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Eq + Hash,
        K: Borrow<Q>,
    {
        let bucket_idx = self.locate_bucket(key);
        let bucket = self
            .buckets
            .get(bucket_idx)
            .expect("locate_bucket() returns a wrong index");

        let idx = bucket.keys.iter().position(|item| item.borrow() == key)?;

        Some(&bucket.values[idx])
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let bucket_idx = self.locate_bucket(key);
        let bucket = self
            .buckets
            .get_mut(bucket_idx)
            .expect("locate_bucket() returns a wrong index");

        let idx = bucket.keys.iter().position(|item| item.borrow() == key)?;

        Some(&mut bucket.values[idx])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_without_split_works() {
        let mut map = HashMap::new();
        assert_eq!(map.insert(1, 1), None);

        assert_eq!(map.get(&1), Some(&1));
    }

    #[test]
    fn insert_duplicate_items() {
        let mut map = HashMap::new();
        assert_eq!(map.insert(1, 1), None);
        assert_eq!(map.insert(1, 1), Some(1));

        assert_eq!(map.get(&1), Some(&1));
    }
}
