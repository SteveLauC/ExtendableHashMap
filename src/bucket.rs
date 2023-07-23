use std::ops::RangeInclusive;

pub(crate) const BUCKET_CAP: usize = 3;

/// Bucket, where data is actually stored.
#[derive(Debug)]
pub(crate) struct Bucket<K, V> {
    /// Bits that are unique to this bucket.
    ///
    /// # Weight
    /// Say the global depth is `i`, weight of `bits[index]` is
    /// `2^(i-index-1)`.
    ///
    /// # Functionality of this field
    /// When updating bucket pointers in `directory`, we need to do
    /// bit-string match to find the corresponding bucket, string
    /// match is slow, we use numeric value for a faster lookup.
    ///
    /// # Example
    /// Say we have bits `[1]`, and the global depth is `3`, then the
    /// bits are automatically expanded to `[1, 0, 0]`, and thus
    /// has value `4`.
    ///
    /// # local depth
    /// Local depth equals `self.bits.len()`.
    pub(crate) bits: Vec<u8>,
    pub(crate) keys: Vec<K>,
    pub(crate) values: Vec<V>,
}

/// A bucket's value
///
/// Calculated through:
/// 1. Global depth
/// 2. Bucket's `bits`
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BucketValue {
    /// This bucket's local depth equals to the global depth.
    EqualTo(usize),
    /// This bucket's local depth is in this range.
    Range(RangeInclusive<usize>),
}

impl BucketValue {
    pub(crate) fn as_equal_to(&self) -> Option<usize> {
        match self {
            BucketValue::EqualTo(val) => Some(*val),
            _ => None,
        }
    }

    pub(crate) fn as_range(&self) -> Option<&RangeInclusive<usize>> {
        match self {
            BucketValue::Range(val) => Some(val),
            _ => None,
        }
    }

    pub(crate) fn last_half_range(&self) -> Option<RangeInclusive<usize>> {
        match self {
            BucketValue::Range(val) => {
                let start = val.start();
                let end = val.end();
                let half_len = (end - start) / 2;

                Some(RangeInclusive::new(*start + half_len, *end))
            }
            _ => None,
        }
    }
}

impl<K, V> Bucket<K, V> {
    /// Create a bucket with the specified configuration.
    ///
    /// # Panic
    /// All numbers in `bits` should be valid binary numbers, i.e., be
    /// smaller than 2.
    pub(crate) fn new(bits: &[u8]) -> Self {
        // check `bits`
        bits.iter().for_each(|bit| assert!(*bit < 2));

        Self {
            bits: bits.to_vec(),
            keys: Vec::with_capacity(BUCKET_CAP),
            values: Vec::with_capacity(BUCKET_CAP),
        }
    }

    /// Return the bucket's local depth.
    #[inline]
    pub(crate) fn local_depth(&self) -> usize {
        self.bits.len()
    }

    /// Given the global depth, calculate this bucket's value.
    pub(crate) fn value(&self, global_depth: usize) -> BucketValue {
        let local_depth = self.bits.len();
        if local_depth == global_depth {
            assert_eq!(self.bits.len(), global_depth);

            let value: usize = self
                .bits
                .iter()
                .rev()
                .map(|u_8| *u_8 as usize)
                .enumerate()
                .fold(0, |acc, (idx, x)| {
                    acc + (2_usize.pow(
                        idx.try_into().expect("Should be in range of u32?"),
                    )) * x
                });

            BucketValue::EqualTo(value)
        } else {
            assert!(local_depth < global_depth);

            let start: usize =
                self.bits.iter().map(|u_8| *u_8 as usize).enumerate().fold(
                    0,
                    |acc, (idx, x)| {
                        acc + (2_usize.pow((global_depth - idx - 1) as u32)) * x
                    },
                );
            let end: usize = start
                + 2_usize.pow((global_depth - self.bits.len()) as u32)
                - 1;

            BucketValue::Range(RangeInclusive::new(start, end))
        }
    }

    /// Return true if this bucket is full.
    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        assert_eq!(self.keys.len(), self.values.len());

        self.keys.len() == self.values.capacity()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn bucket_new_invalid_bit() {
        Bucket::<(), ()>::new(&[3, 1]);
    }

    #[test]
    fn bucket_value() {
        let bucket: Bucket<(), ()> = Bucket::new(&[1, 1]);

        assert_eq!(
            bucket.value(3),
            BucketValue::Range(RangeInclusive::new(6, 7))
        );
        assert_eq!(bucket.value(2), BucketValue::EqualTo(3));
    }
}
