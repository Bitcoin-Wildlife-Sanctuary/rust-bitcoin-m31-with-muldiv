use bitvm::treepp::*;

const MOD: u32 = (1 << 31) - 1;

pub fn m31_to_n31() -> Script {
    script! {
        { MOD } OP_SUB
    }
}

pub fn n31_to_m31() -> Script {
    script! {
        { MOD } OP_ADD
    }
}

pub fn m31_add_n31() -> Script {
    script! {
        OP_ADD
        m31_adjust
    }
}

pub fn n31_add_m31() -> Script {
    script! {
        OP_ADD
        n31_adjust
    }
}

fn m31_adjust() -> Script {
    script! {
        OP_DUP
        0 OP_LESSTHAN
        OP_IF { MOD } OP_ADD OP_ENDIF
    }
}

fn n31_adjust() -> Script {
    script! {
        OP_DUP
        0 OP_GREATERTHANOREQUAL
        OP_IF { MOD } OP_SUB OP_ENDIF
    }
}

pub fn m31_add() -> Script {
    script! {
        m31_to_n31
        m31_add_n31
    }
}

pub fn n31_add() -> Script {
    script! {
        n31_to_m31
        n31_add_m31
    }
}

pub fn m31_double() -> Script {
    script! {
        OP_DUP
        m31_add
    }
}

pub fn n31_double() -> Script {
    script! {
        OP_DUP
        n31_add
    }
}

pub fn m31_sub() -> Script {
    script! {
        OP_SUB
        m31_adjust
    }
}

pub fn n31_sub() -> Script {
    script! {
        OP_SUB
        n31_adjust
    }
}

pub fn m31_neg() -> Script {
    script! {
        { MOD }
        OP_SWAP
        OP_SUB
    }
}

pub fn n31_neg() -> Script {
    script! {
        { -(MOD as i64) }
        OP_SWAP
        OP_SUB
    }
}

pub fn m31_to_bits() -> Script {
    script! {
        for i in 0..30 {
            OP_DUP
            { 1 << (30 - i) } OP_GREATERTHANOREQUAL
            OP_SWAP OP_OVER
            OP_IF { 1 << (30 - i) } OP_SUB OP_ENDIF
        }
    }
}

pub fn m31_mul() -> Script {
    script! {
        // idea:
        // - split a into a_h and a_l where a_h is the higher 16 bits and a_l is the lower 15 bits
        // - split b into b_h and b_l where b_h is the higher 15 bits and b_l is the lower 16 bits
        //
        // a = a_h * 2^15 + a_l
        // b = b_h * 2^16 + b_l
        //
        // a * b = a_l * b_l + (a_h * b_l + a_l * b_h * 2) * 2^15 + a_h * b_h * 2^31
        //       = a_l * b_l + (a_h * b_l + a_l * b_h * 2) * 2^15 + a_h * b_h
        //       = (a_l * b_l + a_h * b_h) + (a_h * b_l + a_l * b_h * 2) * 2^15
        //
        // idea: compute a_l * b_l, a_h * b_h, and perform an addition with modular reduction
        //
        // compute a_h * b_l, a_h * b_h, and compute (a_h * b_l + a_l * b_h * 2) also with a modular reduction
        //
        // here, computing a_h * b_l can be a little bit challenging because both are 16 bits and their multiplication
        // may overflow. To handle that, we divide a_h by 2 while keeping its last bit.
        //
        // split (a_h * b_l + a_l * b_h * 2) into c_h * 2^16 + c_l
        // where c_h is the higher 15 bits and c_l is the lower 16 bits
        //
        // (c_h * 2^16 + c_l) * 2^15 = c_h + c_l * 2^15
        //
        // multiply c_l with 2^15 and add it to the rest with modular reduction

        OP_SWAP

        // split `a`
        OP_DUP
        { 1 << 15 } OP_DIV
        OP_DUP
        { 1 << 15 } OP_MUL
        OP_ROT OP_SWAP OP_SUB
        // current stack: b a_h a_l

        // split `b`
        OP_ROT
        OP_DUP
        { 1 << 16 } OP_DIV
        OP_DUP
        { 1 << 16 } OP_MUL
        OP_ROT OP_SWAP OP_SUB
        // current stack: a_h a_l b_h b_l

        // compute a_h * b_h
        3 OP_PICK
        2 OP_PICK
        OP_MUL OP_TOALTSTACK

        // compute a_l * b_l
        2 OP_PICK
        OP_OVER
        OP_MUL OP_TOALTSTACK

        // compute a_h * b_l
        3 OP_ROLL

        // split a_h = a_h' * 2 + a_lsb
        OP_DUP
        2 OP_DIV
        OP_DUP
        2 OP_MUL
        OP_ROT OP_SWAP OP_SUB

        OP_IF
            OP_OVER
        OP_ELSE
            { 0 }
        OP_ENDIF
        OP_TOALTSTACK
        OP_MUL m31_double OP_TOALTSTACK

        // compute a_l * b_h * 2
        OP_MUL
        m31_double

        // compute c = a_l * b_h * 2 + a_h * b_l
        OP_FROMALTSTACK m31_add
        OP_FROMALTSTACK m31_add

        // split c = c_h * 2^16 + c_l
        OP_DUP
        { 1 << 16 } OP_DIV
        OP_DUP
        { 1 << 16 } OP_MUL
        OP_ROT OP_SWAP OP_SUB

        // stack: c_h c_l
        // altstack: a_h * b_h   a_l * b_l

        { 1 << 15 } OP_MUL
        m31_add
        OP_FROMALTSTACK
        m31_add
        OP_FROMALTSTACK
        m31_add
    }
}

#[cfg(test)]
mod test {
    use bitvm::treepp::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    use super::*;

    #[test]
    fn test_m31_add() {
        let mut prng = ChaCha20Rng::seed_from_u64(0u64);
        eprintln!("m31 add: {}", m31_add().len());

        for _ in 0..100 {
            let a: u32 = prng.gen();
            let b: u32 = prng.gen();

            let a_m31 = a % MOD;
            let b_m31 = b % MOD;
            let sum_m31 = (a_m31 + b_m31) % MOD;

            let script = script! {
                { a_m31 }
                { b_m31 }
                m31_add
                { sum_m31 }
                OP_EQUAL
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_m31_sub() {
        let mut prng = ChaCha20Rng::seed_from_u64(0u64);
        eprintln!("m31 sub: {}", m31_sub().len());

        for _ in 0..100 {
            let a: u32 = prng.gen();
            let b: u32 = prng.gen();

            let a_m31 = a % MOD;
            let b_m31 = b % MOD;
            let diff_m31 = (MOD + a_m31 - b_m31) % MOD;

            let script = script! {
                { a_m31 }
                { b_m31 }
                m31_sub
                { diff_m31 }
                OP_EQUAL
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_m31_to_bits() {
        let mut prng = ChaCha20Rng::seed_from_u64(0u64);

        for _ in 0..100 {
            let a: u32 = prng.gen();
            let m31 = a % MOD;

            let mut bits = vec![];
            let mut cur = m31;
            for _ in 0..31 {
                bits.push(cur % 2);
                cur >>= 1;
            }
            assert_eq!(cur, 0);

            let script = script! {
                { m31 }
                m31_to_bits
                for i in 0..30 {
                    { bits[i as usize] } OP_EQUALVERIFY
                }
                { bits[30] } OP_EQUAL
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_m31_mul() {
        let mut prng = ChaCha20Rng::seed_from_u64(6u64);
        eprintln!("m31 mul: {}", m31_mul().len());

        for _ in 0..100 {
            let a: u32 = prng.gen();
            let b: u32 = prng.gen();

            let a_m31 = a % MOD;
            let b_m31 = b % MOD;
            let prod_m31 = ((((a_m31 as u64) * (b_m31 as u64)) % (MOD as u64)) & 0xffffffff) as u32;

            let script = script! {
                { a_m31 }
                { b_m31 }
                m31_mul
                { prod_m31 }
                OP_EQUAL
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_m31_neg() {
        let mut prng = ChaCha20Rng::seed_from_u64(6u64);
        eprintln!("m31 neg: {}", m31_neg().len());

        for _ in 0..100 {
            let a: u32 = prng.gen();

            let a_m31 = a % MOD;
            let b_m31 = MOD - a_m31;

            let script = script! {
                { a_m31 }
                m31_neg
                { b_m31 }
                OP_EQUAL
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}
