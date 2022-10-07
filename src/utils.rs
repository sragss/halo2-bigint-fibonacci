use std::str::FromStr;

use num_bigint::BigUint;

/// 0-indexed fib calc
pub fn fib_calc(n: usize) -> u128 {
    assert!(n <= 185); // Otherwise u128 overflow

    let init_a = 0;
    let init_b = 1;
    let mut prev = init_b;
    let mut sum = init_a + init_b;

    for _ in 2..n {
        let tmp = sum;
        sum += prev;
        prev = tmp;
    }

    sum
}

pub fn big_fib_calc(n: usize) -> BigUint {
    let init_a = BigUint::from(0u8);
    let init_b = BigUint::from(1u8);
    let mut prev = init_b.clone();
    let mut sum = init_a + init_b;

    for _ in 2..n {
        let tmp = sum.clone();
        sum += prev;
        prev = tmp;
    }

    sum
}

#[test]
fn big_fib_calc_no_overflow() {
    let expect = BigUint::from_str("222232244629420445529739893461909967206666939096499764990979600").unwrap();
    assert_eq!(big_fib_calc(300), expect);
}