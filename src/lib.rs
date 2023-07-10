#![feature(vec_push_within_capacity)]

mod util;

use std::{cell::RefCell, hash::Hash, ops::RangeInclusive, rc::Rc};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

/// A set backed by Extendable Hashing.
#[derive(Debug)]
pub struct HashSet<T> {
    /// Global depth
    global_depth: usize,
    /// Directory entries
    directories: Vec<DirectoryEntry<T>>,
}

/// Directory Entry
#[derive(Debug)]
struct DirectoryEntry<T> {
    // TODO: maybe this field is useless?
    /// `id` can be seen as the index for this entry in directories.
    id: usize,
    /// `bucket` pointed by this directory entry.
    ///
    /// A bucket will have more than one pointer pointing to it if it `local_depth`
    /// is smaller than `global depth` so that `Rc` is needed here.
    bucket: Option<Rc<RefCell<Bucket<T>>>>,
}

impl<T> DirectoryEntry<T> {
    /// Create a `DirectoryEntry` with `bucket` field uninitialized.
    #[inline]
    fn new(id: usize) -> Self {
        Self { id, bucket: None }
    }

    /// Set the `bucket` field.
    ///
    /// # Panic
    /// Panic if this `DirectoryEntry`'s `bucket` field has already been set.
    #[inline]
    fn budget(&mut self, bucket: Bucket<T>) {
        assert!(self.bucket.is_none());

        self.bucket = Some(Rc::new(RefCell::new(bucket)));
    }
}

/// Bucket, where data is actually stored.
#[derive(Debug)]
struct Bucket<T> {
    /// Bits that are unique to this bucket.
    ///
    /// # Weight
    /// Say the global depth is `i`, weight of `bits[index]` is `2^(i-index-1)`.
    ///
    /// # Functionality of this field
    /// When updating bucket pointers in `directory`, we need to do bit-string
    /// match to find the corresponding bucket, string match is slow, we use
    /// numeric value for a faster lookup.
    ///
    /// # Example
    /// Say we have bits `[1]`, and the global depth is `3`, then the bits are
    /// automatically expanded to `[1, 0, 0]`, and thus has value `4`.
    bits: Vec<u8>,
    /// Local depth this bucket has.
    local_depth: usize,
    /// The actual data.
    ///
    /// They are not sorted.
    data: Vec<T>,
}

/// A bucket's value
///
/// Calculated through:
/// 1. Global depth
/// 2. Bucket's `bits`
#[derive(Debug, PartialEq, Eq)]
enum BucketValue {
    /// This bucket's local depth equals to the global depth.
    EqualTo(usize),
    /// This bucket's local depth is in this range.
    Range(RangeInclusive<usize>),
}

impl<T> Bucket<T> {
    /// Create a bucket with the specified configuration.
    ///
    /// # Panic
    /// All numbers in `bits` should be valid binary numbers, i.e., be smaller than 2,
    /// `bucket_size` should be greater than 0, or this function will panic.
    fn new(local_depth: usize, bucket_size: usize, bits: &[u8]) -> Self {
        // check `bits`
        bits.iter().for_each(|bit| assert!(*bit < 2));
        // check bucket size
        assert!(bucket_size > 0);

        Self {
            local_depth,
            data: Vec::with_capacity(bucket_size),
            bits: bits.to_vec(),
        }
    }

    /// Given the global depth, calculate this bucket's value.
    fn value(&self, global_depth: usize) -> BucketValue {
        if self.local_depth == global_depth {
            assert_eq!(self.bits.len(), global_depth);

            let value: usize = self
                .bits
                .iter()
                .rev()
                .map(|u_8| *u_8 as usize)
                .enumerate()
                .fold(0, |acc, (idx, x)| {
                    acc + (2_usize.pow(idx.try_into().expect("Should be in range of u32?"))) * x
                });

            BucketValue::EqualTo(value)
        } else {
            assert!(self.local_depth < global_depth);

            let start: usize = self
                .bits
                .iter()
                .map(|u_8| *u_8 as usize)
                .enumerate()
                .fold(0, |acc, (idx, x)| {
                    acc + (2_usize.pow((global_depth - idx - 1) as u32)) * x
                });
            let end: usize = start + 2_usize.pow((global_depth - self.bits.len()) as u32) - 1;

            BucketValue::Range(RangeInclusive::new(start, end))
        }
    }
}

impl<T> HashSet<T> {
    /// Create an empty `HashSet`.
    pub fn new() -> Self {
        unimplemented!()
    }
}

impl<T: Hash> HashSet<T> {
    fn locate_bucket(&self, value: &T) -> usize {
        let mut default_hasher = DefaultHasher::new();
        value.hash(&mut default_hasher);
        let hash_res = default_hasher.finish();


        unimplemented!()
    }

    fn split(&mut self, bucket: ()) {
        unimplemented!()
    }


    pub fn insert(&mut self, value: T) {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn bucket_new_invalid_bit() {
        Bucket::<()>::new(3, 1, &[3, 1]);
    }

    #[test]
    #[should_panic]
    fn bucket_new_zero_size() {
        Bucket::<()>::new(3, 0, &[1]);
    }

    #[test]
    fn bucket_value() {
        let bucket: Bucket<()> = Bucket::new(2, 1, &[1, 1]);

        assert_eq!(
            bucket.value(3),
            BucketValue::Range(RangeInclusive::new(6, 7))
        );
        assert_eq!(bucket.value(2), BucketValue::EqualTo(3));
    }
}
