use crate::{
    bucket::{
        Bucket,
        BucketValue::{EqualTo, Range},
        BUCKET_CAP,
    },
    util::{bits_to_value, get_first_n_bits},
};
use std::{
    borrow::Borrow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// A map backed by Extendable Hashing.
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
    /// Locate the bucket where `value` will go.
    fn locate_bucket<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash,
    {
        let mut default_hasher = DefaultHasher::new();
        key.hash(&mut default_hasher);
        let hash_res = default_hasher.finish();

        // Use the reverse last `self.global` bits
        //
        // NOTE: we need to ensure the following guarantee:
        // Say the global depth is 1, and the hashing bits are `[0]`, after
        // we increment the global depth to 2, the hashing bits have to be
        // either `[0, 0]` or `[0, 1]`
        let first_bits = get_first_n_bits(self.global_depth, hash_res);
        let directory_idx = bits_to_value(first_bits.as_slice());

        self.directories[directory_idx]
    }

    /// Split a bucket.
    ///
    /// Under awful cases, this function will be called recursively.
    fn split(&mut self, key: K, value: V, bucket_to_split: usize) {
        let mut_ref_bucket = self.buckets.get_mut(bucket_to_split).unwrap();

        let old_local_depth = mut_ref_bucket.local_depth();
        let old_global_depth = self.global_depth;
        assert!(old_local_depth <= old_global_depth);

        let bucket_value = mut_ref_bucket.value(old_global_depth);
        let mut bucket_slice = mut_ref_bucket.bits.clone();
        mut_ref_bucket.bits.push(0);
        bucket_slice.push(1);
        let new_bucket = Bucket::new(bucket_slice.as_slice());
        let new_bucket_idx = self.buckets.len();
        self.buckets.push(new_bucket);

        if old_local_depth < old_global_depth {
            let last_half_directory_indexes =
                // this bucket_value needs to be calculated before incrementing
                // the local depth because we are redistributing the old pointers.
                bucket_value.last_half_range().unwrap();

            // redistribute pointers
            for idx in last_half_directory_indexes {
                self.directories[idx] = new_bucket_idx;
            }
        } else {
            self.global_depth += 1;
            for _ in 0..self.directories.len() {
                self.directories.push(0);
            }

            // Redistribute directory pointers
            //
            // As you can see, we need to redistribute *all* the pointers. When
            // expanding the directory entry, we choose to *append* instead of
            // *insert*ing as inserting to an array is not efficient
            //
            // [0, 1] => [00, 01, 10, 11] ([0, 1] => [0, 1, 2, 3])
            //
            // Appending is efficient but it will invalidate the pointers stored
            // in the old directory entries, and thus we have to redistribute
            // them all.
            //
            // What about using a linked list, well, we need fast random access
            // when locating a bucket.
            for (bucket_idx, bucket) in self.buckets.iter().enumerate() {
                let bucket_value = bucket.value(self.global_depth);

                match bucket_value {
                    EqualTo(idx) => self.directories[idx] = bucket_idx,
                    Range(range) => {
                        for idx in range {
                            self.directories[idx] = bucket_idx;
                        }
                    }
                }
            }
        }

        // rehashing the existing items
        let items_need_rehash = self.buckets[bucket_to_split]
            .data
            .drain(..)
            .collect::<Vec<(K, V)>>();
        for (k, v) in items_need_rehash {
            let idx = self.locate_bucket(k.borrow());
            assert!(idx == bucket_to_split || idx == new_bucket_idx);

            self.buckets[idx].data.push((k, v));
        }

        // after split, try inserting the new item again
        let idx = self.locate_bucket(key.borrow());
        // let's do split again.
        if self.buckets[idx].is_full() {
            self.split(key, value, idx);
        } else {
            if let Err(_) =
                self.buckets[idx].data.push_within_capacity((key, value))
            {
                panic!("push_within_capacity failed")
            }
        }
    }

    /// Insert `value` to this set.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Eq,
    {
        let bucket_idx = self.locate_bucket(key.borrow());
        let mut_ref_bucket = self.buckets.get_mut(bucket_idx).unwrap();

        // Check existence
        if mut_ref_bucket.contains(&key) {
            return Some(value);
        }

        if !mut_ref_bucket.is_full() {
            if let Err(_) =
                mut_ref_bucket.data.push_within_capacity((key, value))
            {
                panic!("push_within_capacity failed")
            }
        } else {
            self.split(key, value, bucket_idx);
        }
        self.len += 1;

        None
    }

    /// Returns a reference to the value corresponding to the key.
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

        bucket
            .data
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|kv| &kv.1)
    }

    /// Returns a mutable reference to the value corresponding to the key.
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

        bucket
            .data
            .iter_mut()
            .find(|(k, _)| k.borrow() == key)
            .map(|kv| &mut kv.1)
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

    #[test]
    fn insert_1000_items() {
        let mut map = HashMap::new();
        for i in 0..1000 {
            assert_eq!(map.insert(i, i), None);
        }

        assert_eq!(map.len(), 1000);
    }
}
