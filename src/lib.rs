#![allow(unexpected_cfgs)]
#![allow(unused)] /* jesus christ */
#![cfg_attr(
    target_os = "solana",
    feature(asm_experimental_arch),
    feature(asm_goto)
)]

use std::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, log::sol_log_64, msg, pubkey::Pubkey, syscalls::sol_log_pubkey,
};

/// IF YOU USE THIS PLS REMEMBER TO USE THIS AS HEAPSTART
///
/// YOU WILL HAVE TO REWRITE ALLOCATOR
const fn heap_start(num_accounts: usize) -> usize {
    0x300000000 + num_accounts * 8
}

const ACCOUNT_INFO_SIZE: usize = 88;
const MAX_PERMITTED_ACCOUNT_DATA_SIZE: usize = 10240;
const RENT_EPOCH_SIZE: usize = 8;
const TOTAL_ACCOUNT_DATA_TO_SKIP: usize =
    ACCOUNT_INFO_SIZE + MAX_PERMITTED_ACCOUNT_DATA_SIZE + RENT_EPOCH_SIZE + 7;
const ACCOUNTS_PTR: usize = 0x300000000;

#[no_mangle]
pub unsafe extern "C" fn entrypoint(mut input: *mut u8) -> u32 {
    let mut num_accounts = MaybeUninit::<usize>::uninit();
    #[cfg(target_os = "solana")]
    core::arch::asm!(
        // Load num accounts
        "ldxdw r5, [r1 + 0]",

        // Initialize accounts cursor and make a copy for duplicates
        "lddw r7, {accounts_ptr}",
        "mov64 r4, r7",

        inout("r1") input,
        out("r5") num_accounts,
        accounts_ptr = const ACCOUNTS_PTR,
        options(nostack),
    );
    let num_accounts = num_accounts.assume_init();

    use asmr_macro::{binary_tree_dispatch, entrypoint_process, entrypoint_process_batched};

    if num_accounts > 16 {
        return u32::MAX;
    } else {
        #[cfg(target_os = "solana")]
        {
            let x = {
                #[inline(always)]
                |r1: *mut u8| {
                    binary_tree_dispatch!(
                        num_accounts,
                        16,
                        entrypoint_process,
                        9,
                        entrypoint_process_batched
                    )
                }
            };
            x(input);
        }
    }

    #[cfg(target_os = "solana")]
    core::arch::asm!(
        // Finished
        "add64 r1, 16",

        inout("r1") input,
        options(nostack),
    );

    let instruction_data_len = *(input as *const u64) as usize;
    // input = input.add(core::mem::size_of::<u64>());

    let data = core::slice::from_raw_parts(input.sub(8), instruction_data_len);
    input = input.add(instruction_data_len);

    let program_id: &Pubkey = &*(input as *const Pubkey);

    let accounts = core::slice::from_raw_parts(ACCOUNTS_PTR as *const AccountInfo, num_accounts);

    // sol_log_64(data.len() as u64, accounts.len() as u64, 0, 0, 0);

    process(program_id, accounts, data)
}

#[inline(always)]
#[allow(unused)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> u32 {
    0
}
