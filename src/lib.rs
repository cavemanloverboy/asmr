#![allow(unexpected_cfgs)]
#![allow(unused)] /* jesus christ */
#![cfg_attr(target_os = "solana", feature(asm_experimental_arch, asm_const))]

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
    ACCOUNT_INFO_SIZE + MAX_PERMITTED_ACCOUNT_DATA_SIZE + RENT_EPOCH_SIZE;
const ACCOUNTS_PTR: usize = 0x300000000;

#[no_mangle]
pub unsafe extern "C" fn entrypoint(mut input: *mut u8) -> u32 {
    let mut num_accounts = MaybeUninit::<usize>::uninit();
    #[cfg(target_os = "solana")]
    core::arch::asm!(
        // Load num accounts (r9 will be used to move into num_accounts stack..)
        "ldxdw r9, [r1 + 0]",
        "mov64 r5, r9",
        // Advance input cursor by num accounts
        "add64 r1, 8",
        // Initialize accounts cursor
        "lddw r7, {accounts_ptr}",

        // first account is GUARANTEED to be nondup so we don't need dup check
        // inline nondup case
        "jeq r5, 0, 6f", // still need to check there's at least one account
        /* START INLINE */
        // Store account ptr and load account data len
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        // Advance input cursor by static data, account data, and round up to next 8
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "add64 r1, 7",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        // Decrement account counter
        "sub64 r5, 1",
        /* END INLINE (DON'T NEED JUMP) */

        // Check if finished
        "jeq r5, 0, 6f",

        "2:",
        // Otherwise, increment account cursor, load dup marker, jump to dup if dup
        "add64 r7, 8",
        "ldxb r6, [r1 + 0]",
        "jne r6, 255, 5f",

        // Non-duplicate account case
        "3:",
        // Store account ptr and load account data len
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        // Advance input cursor by static data, account data, and round up to next 8
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "add64 r1, 7",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        // Decrement account counter and go back to check if done
        "sub64 r5, 1",
        "jne r5, 0, 2b",
        "ja 6f",


        // Duplicate account case
        "5:",
        // Calculate index, load account into r6
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        // Store in r7 and advance input cursor
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        // Decrement account counter and go back to top if not done
        "sub64 r5, 1",
        "jne r5, 0, 2b",

        // Finished
        "6:",


        inout("r1") input,
        out("r9") num_accounts,
        accounts_ptr = const ACCOUNTS_PTR,
        account_total = const TOTAL_ACCOUNT_DATA_TO_SKIP,
        options(nostack),
    );

    let instruction_data_len = *(input as *const u64) as usize;
    input = input.add(core::mem::size_of::<u64>());

    let data = core::slice::from_raw_parts(input, instruction_data_len);
    input = input.add(instruction_data_len);

    let program_id: &Pubkey = &*(input as *const Pubkey);

    let accounts = core::slice::from_raw_parts(
        ACCOUNTS_PTR as *const AccountInfo,
        num_accounts.assume_init(),
    );

    // sol_log_64(data.len() as u64, accounts.len() as u64, 0, 0, 0);

    process(program_id, accounts, data)
}

#[inline(always)]
#[allow(unused)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> u32 {
    0
}
