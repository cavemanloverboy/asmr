use litesvm::LiteSVM;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

fn main() {
    let program_id = Pubkey::new_unique();
    let mut svm = LiteSVM::new()
        .with_sigverify(false)
        .with_blockhash_check(false)
        .with_transaction_history(0);
    svm.add_program_from_file(program_id, "target/deploy/asmr.so")
        .unwrap();
    let payer = Pubkey::new_unique();
    svm.airdrop(&payer, 1_000_000_000).unwrap();

    for n in [0, 1, 2, 4, 8, 16, 32, 64] {
        println!("\nFor {n} accounts:");
        // nondup;
        let ixn = Instruction {
            program_id,
            accounts: (0..n)
                .map(|_| AccountMeta::new_readonly(Pubkey::new_unique(), false))
                .collect(),
            data: vec![1, 2, 3],
        };
        let msg = Message::new(&[ixn], Some(&payer));
        let txn = Transaction::new_unsigned(msg);

        match svm.send_transaction(txn) {
            Ok(res) => {
                for log in res.logs {
                    println!("    {log}")
                }
            }
            Err(e) => {
                println!("ERROR:");
                for log in e.meta.logs {
                    println!("    {log}")
                }
            }
        }
    }

    for n in [2, 4, 8, 16, 32, 64] {
        println!("\nFor {n} dup accounts:");
        // nondup;
        let ixn = Instruction {
            program_id,
            accounts: vec![AccountMeta::new_readonly(Pubkey::new_unique(), false); n],
            data: vec![1, 2, 3],
        };
        let msg = Message::new(&[ixn], Some(&payer));
        let txn = Transaction::new_unsigned(msg);

        match svm.send_transaction(txn) {
            Ok(res) => {
                for log in res.logs {
                    println!("    {log}")
                }
            }
            Err(e) => {
                println!("ERROR:");
                for log in e.meta.logs {
                    println!("    {log}")
                }
            }
        }
    }
}
