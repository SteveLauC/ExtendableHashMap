/// Get the first `n` bits of `num`
pub(crate) fn get_first_n_bits(n: usize, mut num: u64) -> Vec<usize> {
    let mut ret = Vec::with_capacity(n);
    let val_msb = 2_u64.pow(u64::BITS - 1);

    for _ in 0..n {
        ret.push(if num >= val_msb { 1 } else { 0 });
        num <<= 1;
    }

    ret
}

/// Convert `bits` to its value
///
/// # Example
///
/// [0, 0] => (0 * 2^0) + (0*2^1) = 0
pub(crate) fn bits_to_value(bits: &[usize]) -> usize {
    bits.iter()
        .rev()
        .enumerate()
        .fold(0, |acc, (idx, bit)| acc + (bit * 2_usize.pow(idx as _)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_first_n_bits_works() {
        let bits = get_first_n_bits(u64::BITS as _, u64::MAX);

        bits.iter().for_each(|bit| assert_eq!(*bit, 1));

        let bits = get_first_n_bits(u64::BITS as _, u64::MIN);

        bits.iter().for_each(|bit| assert_eq!(*bit, 0));
    }

    #[test]
    fn bits_to_value_works() {
        let bits = [1, 1, 0];
        assert_eq!(bits_to_value(&bits), 6);
    }
}
