/// Get the first `n` bits of `num`
pub(crate) fn get_first_n_bits(n: usize, mut num: u64) -> Vec<u8> {
    let mut ret = Vec::with_capacity(n);
    let val_msb = 2_u64.pow(u64::BITS - 1);

    for _ in 0..n {
        ret.push(if num >= val_msb { 1 } else { 0 });
        num <<= 1;
    }

    ret
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
}
