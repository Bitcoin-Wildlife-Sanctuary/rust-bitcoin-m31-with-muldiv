use crate::treepp::*;
use crate::{m31_add, m31_mul, m31_sub};

// Input: A1 B1 A2 B2
// Output:
//      A1B2 + A2B1
//      B1B2 - A1A2
pub fn karatsuba_small() -> Script {
    script! {
        OP_OVER 4 OP_PICK
        m31_mul
        OP_TOALTSTACK
        OP_DUP
        3 OP_PICK
        m31_mul
        OP_TOALTSTACK
        m31_add
        OP_TOALTSTACK
        m31_add
        OP_FROMALTSTACK
        m31_mul
        OP_FROMALTSTACK
        OP_FROMALTSTACK
        OP_2DUP
        m31_add
        3 OP_ROLL
        OP_SWAP
        m31_sub
        OP_TOALTSTACK
        m31_sub
        OP_FROMALTSTACK
        OP_SWAP
    }
}

// Input:
//      A1 B1 C1 D1
//      A2 B2 C2 D2
// Output:
//      (A1, B1) * (A2, B2) - 2 elements
//      (A1, B1) * (C2, D2) + (A2, B2) * (C1, D1) - 2 elements
//      (C1, D1) * (C2, D2) - 2 elements
pub fn karatsuba_big() -> Script {
    script! {
        7 OP_PICK
        7 OP_PICK
        5 OP_PICK
        5 OP_PICK
        karatsuba_small
        OP_TOALTSTACK
        OP_TOALTSTACK
        OP_2DUP
        7 OP_PICK
        7 OP_PICK
        karatsuba_small
        OP_TOALTSTACK
        OP_TOALTSTACK
        OP_ROT
        m31_add
        OP_TOALTSTACK
        m31_add
        OP_TOALTSTACK
        OP_ROT
        m31_add
        OP_TOALTSTACK
        m31_add
        OP_FROMALTSTACK
        OP_FROMALTSTACK
        OP_FROMALTSTACK
        karatsuba_small
        OP_FROMALTSTACK
        OP_FROMALTSTACK
        OP_FROMALTSTACK
        OP_FROMALTSTACK
        5 OP_ROLL
        2 OP_PICK
        5 OP_PICK
        m31_add
        m31_sub
        5 OP_ROLL
        2 OP_PICK
        5 OP_PICK
        m31_add
        m31_sub
        5 OP_ROLL
        5 OP_ROLL
    }
}

#[cfg(test)]
mod test {
    use super::{karatsuba_big, karatsuba_small};
    use crate::treepp::*;
    use core::ops::{Add, Mul, Sub};
    use p3_field::extension::Complex;
    use p3_field::PrimeField32;
    use p3_mersenne_31::Mersenne31 as P3M31;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_small_karatsuba() {
        let mut prng = ChaCha20Rng::seed_from_u64(0u64);

        let a1: P3M31 = prng.gen();
        let b1: P3M31 = prng.gen();
        let a2: P3M31 = prng.gen();
        let b2: P3M31 = prng.gen();

        let first = a1.mul(b2).add(a2.mul(b1));
        let second = b1.mul(b2).sub(a1.mul(a2));

        let script = script! {
            { a1.as_canonical_u32() } { b1.as_canonical_u32() } { a2.as_canonical_u32() } { b2.as_canonical_u32() }
            karatsuba_small
            { second.as_canonical_u32() }
            OP_EQUALVERIFY
            { first.as_canonical_u32() }
            OP_EQUAL
        };
        let exec_result = execute_script(script);
        assert!(exec_result.success);
    }

    #[test]
    fn test_big_karatsuba() {
        let mut prng = ChaCha20Rng::seed_from_u64(0u64);

        let a1: P3M31 = prng.gen();
        let b1: P3M31 = prng.gen();
        let c1: P3M31 = prng.gen();
        let d1: P3M31 = prng.gen();

        let a2: P3M31 = prng.gen();
        let b2: P3M31 = prng.gen();
        let c2: P3M31 = prng.gen();
        let d2: P3M31 = prng.gen();

        let group1_first = a1.mul(b2).add(a2.mul(b1));
        let group1_second = b1.mul(b2).sub(a1.mul(a2));

        let group3_first = c1.mul(d2).add(c2.mul(d1));
        let group3_second = d1.mul(d2).sub(c1.mul(c2));

        let group2_first = a1.mul(d2).add(b1.mul(c2)).add(a2.mul(d1)).add(b2.mul(c1));
        let group2_second = b1.mul(d2).add(b2.mul(d1)).sub(a1.mul(c2).add(a2.mul(c1)));

        let script = script! {
            { a1.as_canonical_u32() } { b1.as_canonical_u32() } { c1.as_canonical_u32() } { d1.as_canonical_u32() }
            { a2.as_canonical_u32() } { b2.as_canonical_u32() } { c2.as_canonical_u32() } { d2.as_canonical_u32() }
            karatsuba_big
            { group3_second.as_canonical_u32() }
            OP_EQUALVERIFY
            { group3_first.as_canonical_u32() }
            OP_EQUALVERIFY
            { group2_second.as_canonical_u32() }
            OP_EQUALVERIFY
            { group2_first.as_canonical_u32() }
            OP_EQUALVERIFY
            { group1_second.as_canonical_u32() }
            OP_EQUALVERIFY
            { group1_first.as_canonical_u32() }
            OP_EQUAL
        };
        let exec_result = execute_script(script);
        assert!(exec_result.success);
    }

    #[test]
    fn test_small_karatsuba_consistency() {
        let mut rng = ChaCha20Rng::seed_from_u64(0u64);

        let a: Complex<p3_mersenne_31::Mersenne31> = rng.gen();
        let b: Complex<p3_mersenne_31::Mersenne31> = rng.gen();
        let c = a.mul(b);

        let script = script! {
            { a.imag().as_canonical_u32() } { a.real().as_canonical_u32() }
            { b.imag().as_canonical_u32() } { b.real().as_canonical_u32() }
            karatsuba_small
            { c.real().as_canonical_u32() }
            OP_EQUALVERIFY
            { c.imag().as_canonical_u32() }
            OP_EQUAL
        };
        let exec_result = execute_script(script);
        assert!(exec_result.success);
    }
}
