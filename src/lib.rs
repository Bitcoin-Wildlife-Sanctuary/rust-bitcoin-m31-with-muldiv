mod m31;
pub use m31::*;

mod qm31;
pub use qm31::*;

mod karatsuba_complex;

pub(crate) mod treepp {
    pub use bitcoin::ScriptBuf as Script;
    pub use bitcoin_script::{define_pushable, script};

    #[cfg(test)]
    pub fn execute_script(script: Script) -> bitcoin_scriptexec::ExecuteInfo {
        execute_script_with_witness(script, vec![])
    }

    #[cfg(test)]
    pub fn execute_script_with_witness(
        script: Script,
        witness: Vec<Vec<u8>>,
    ) -> bitcoin_scriptexec::ExecuteInfo {
        use bitcoin::hashes::Hash;
        use bitcoin::{TapLeafHash, Transaction};
        use bitcoin_scriptexec::{Exec, ExecCtx, ExecuteInfo, FmtStack, Options, TxTemplate};
        let mut exec = Exec::new(
            ExecCtx::Tapscript,
            Options::default_with_mul_div(),
            TxTemplate {
                tx: Transaction {
                    version: bitcoin::transaction::Version::TWO,
                    lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
                    input: vec![],
                    output: vec![],
                },
                prevouts: vec![],
                input_idx: 0,
                taproot_annex_scriptleaf: Some((TapLeafHash::all_zeros(), None)),
            },
            script,
            witness,
        )
        .expect("error creating exec");

        loop {
            if exec.exec_next().is_err() {
                break;
            }
        }
        let res = exec.result().unwrap();
        ExecuteInfo {
            success: res.success,
            error: res.error.clone(),
            last_opcode: res.opcode,
            final_stack: FmtStack(exec.stack().clone()),
            remaining_script: exec.remaining_script().to_asm_string(),
            stats: exec.stats().clone(),
        }
    }

    define_pushable!();
}
