use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Expr, Ident, LitInt, Token};

/// Input: binary_tree_dispatch!(expr, max, callee)
struct DispatchInput {
    value_expr: Expr,
    _comma1: Token![,],
    max: LitInt,
    _comma2: Token![,],
    callee: Ident,
    _comma3: Token![,],
    transition: LitInt,
    _comma4: Token![,],
    callee_upper: Ident,
}

impl Parse for DispatchInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DispatchInput {
            value_expr: input.parse()?,
            _comma1: input.parse()?,
            max: input.parse()?,
            _comma2: input.parse()?,
            callee: input.parse()?,
            _comma3: input.parse()?,
            transition: input.parse()?,
            _comma4: input.parse()?,
            callee_upper: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn binary_tree_dispatch(input: TokenStream) -> TokenStream {
    let DispatchInput {
        value_expr,
        _comma1,
        max,
        _comma2,
        callee,
        _comma3,
        transition,
        _comma4,
        callee_upper,
    } = parse_macro_input!(input as DispatchInput);
    let max_val = max
        .base10_parse::<usize>()
        .expect("Expected integer literal");

    let transition_val = transition
        .base10_parse::<usize>()
        .expect("Expected integer literal");

    fn generate_tree(
        val_expr: &Expr,
        callee: &Ident,
        callee_upper: &Ident,
        transition: usize,
        low: usize,
        high: usize,
    ) -> proc_macro2::TokenStream {
        let range_size = high - low + 1;

        match range_size {
            1 => {
                let call_lit = syn::LitInt::new(&low.to_string(), proc_macro2::Span::call_site());
                if low >= transition {
                    quote! {
                        #callee!(#call_lit)
                    }
                } else {
                    quote! {
                        #callee_upper!(#call_lit)
                    }
                }
            }
            2 => {
                let high_lit = syn::LitInt::new(&high.to_string(), proc_macro2::Span::call_site());
                let low_lit = syn::LitInt::new(&low.to_string(), proc_macro2::Span::call_site());

                // Determine at compile time which macro to call
                let high_call = if high >= transition {
                    quote! { #callee_upper!(#high_lit) }
                } else {
                    quote! { #callee!(#high_lit) }
                };

                let low_call = if low >= transition {
                    quote! { #callee_upper!(#low_lit) }
                } else {
                    quote! { #callee!(#low_lit) }
                };

                quote! {
                    if #val_expr >= #high_lit {
                        #high_call
                    } else {
                        #low_call
                    }
                }
            }
            3 => {
                let high_lit = syn::LitInt::new(&high.to_string(), proc_macro2::Span::call_site());
                let mid_lit =
                    syn::LitInt::new(&(low + 1).to_string(), proc_macro2::Span::call_site());
                let low_lit = syn::LitInt::new(&low.to_string(), proc_macro2::Span::call_site());

                // Determine at compile time which macro to call
                let high_call = if high >= transition {
                    quote! { #callee_upper!(#high_lit) }
                } else {
                    quote! { #callee!(#high_lit) }
                };

                let mid_call = if (low + 1) >= transition {
                    quote! { #callee_upper!(#mid_lit) }
                } else {
                    quote! { #callee!(#mid_lit) }
                };

                let low_call = if low >= transition {
                    quote! { #callee_upper!(#low_lit) }
                } else {
                    quote! { #callee!(#low_lit) }
                };

                quote! {
                    if #val_expr >= #high_lit {
                        #high_call
                    } else if #val_expr == #mid_lit {
                        #mid_call
                    } else {
                        #low_call
                    }
                }
            }
            _ => {
                let mid = (low + high) / 2;
                let mid_lit = syn::LitInt::new(&mid.to_string(), proc_macro2::Span::call_site());

                let left_branch =
                    generate_tree(val_expr, callee, callee_upper, transition, low, mid);
                let right_branch =
                    generate_tree(val_expr, callee, callee_upper, transition, mid + 1, high);

                quote! {
                    if #val_expr > #mid_lit {
                        #right_branch
                    } else {
                        #left_branch
                    }
                }
            }
        }
    }

    let tree = generate_tree(
        &value_expr,
        &callee,
        &callee_upper,
        transition_val,
        0,
        max_val,
    );
    quote!(#tree).into()
}

#[proc_macro]
pub fn entrypoint_process(input: TokenStream) -> TokenStream {
    let total = parse_macro_input!(input as LitInt)
        .base10_parse::<usize>()
        .unwrap();

    let mut output = quote! {
        return;
    };

    for remaining in 1..=total {
        eprintln!("working on {} for {} inlined", remaining, total);
        let block = quote! {

            // not first account
            if const { #remaining != #total } {
                //#[cfg(target_os = "solana")]
                core::arch::asm! {
                    // increment account cursor, load dup marker, jump to dup if dup
                    "add64 r7, 8",
                    "ldxb r3, [r1 + 8]",
                    "jne r3, 255, {dup}",
                    dup = label {
                        unsafe {
                            // DUP
                            //#[cfg(target_os = "solana")]
                            core::arch::asm! {
                                // Calculate index, load account into r3
                                "mul64 r3, 8",
                                "add64 r3, r4",
                                "ldxdw r3, [r3 + 0]",
                                // Store in r7 and advance input cursor
                                "stxdw [r7 + 0], r3",
                                "add64 r1, 8",

                                options(nostack),
                            };

                            // inlined tail call
                            #output
                        }
                    }
                };
            }

            // NONDUP
            //#[cfg(target_os = "solana")]
            core::arch::asm! {
                // Store account ptr and load account data len
                "stxdw [r7 + 0], r1",
                "ldxdw r8, [r1 + 80 + 8]",
                // Advance input cursor by static data, account data, and round up to next 8
                "add64 r1, {account_total}",
                "add64 r1, r8",
                "and64 r1, 0xFFFFFFFFFFFFFFF8",

                account_total = const TOTAL_ACCOUNT_DATA_TO_SKIP,
                options(nostack),
            };

            // inlined tail call
            #output
        };

        output = quote! {{
            #block
        }};
    }

    output = quote! {{
        #output
    }};

    let result = output.into();
    result
}

#[proc_macro]
pub fn entrypoint_process_batched(input: TokenStream) -> TokenStream {
    let total = parse_macro_input!(input as LitInt)
        .base10_parse::<usize>()
        .unwrap();

    let batch_size = 8;
    let num_full_batches = total / batch_size;
    let remainder = total % batch_size;

    let mut output = quote! {};

    // Generate unrolled code for each full batch
    for batch_idx in 0..num_full_batches {
        eprintln!("working on batch {} for {} batched", batch_idx, total);
        let is_first_batch = batch_idx == 0;
        let batch_code = generate_inlined_batch(batch_size, is_first_batch);
        output = quote! {
            #output
            // Batch #batch_idx
            #batch_code
        };
    }

    // Generate code for remainder if any
    if remainder > 0 {
        eprintln!("working on remainder batch for {} batched", total);
        let is_first_batch = num_full_batches == 0;
        let remainder_code = generate_inlined_batch(remainder, is_first_batch);
        output = quote! {
            #output
            // Remainder (#remainder accounts)
            #remainder_code
        };
    }

    let result = quote! {{
        #output
    }};
    result.into()
}

// Generate the fully inlined code for a batch of `count` accounts
fn generate_inlined_batch(count: usize, is_first_batch: bool) -> proc_macro2::TokenStream {
    generate_account_tree(count, count, is_first_batch)
}

// Recursively generate the account processing tree with both dup and nondup branches
fn generate_account_tree(
    remaining: usize,
    total: usize,
    is_first_in_program: bool,
) -> proc_macro2::TokenStream {
    if remaining == 0 {
        // Base case - no more accounts to process in this batch
        return quote! {
            // Batch complete
            return
        };
    }

    let is_first_account = is_first_in_program && remaining == total;
    let next_tree = generate_account_tree(remaining - 1, total, is_first_in_program);

    if is_first_account {
        // First account optimization - no dup check needed
        quote! {
            //#[cfg(target_os = "solana")]
            core::arch::asm! {
                // FIRST ACCOUNT - Store account ptr and load account data len
                "stxdw [r7 + 0], r1",
                "ldxdw r8, [r1 + 80 + 8]",
                // Advance input cursor by static data, account data, and round up to next 8
                "add64 r1, {account_total}",
                "add64 r1, r8",
                "and64 r1, 0xFFFFFFFFFFFFFFF8",
                // Move to next account slot
                "add64 r7, 8",

                account_total = const TOTAL_ACCOUNT_DATA_TO_SKIP,
                options(nostack),
            };

            // Continue with remaining accounts
            #next_tree
        }
    } else {
        // Regular account with dup check - BOTH branches get the full remaining tree
        quote! {
            //#[cfg(target_os = "solana")]
            core::arch::asm! {
                // Load dup marker and check
                "ldxb r3, [r1 + 8]",
                "jne r3, 255, {dup}",
                dup = label {
                    unsafe {
                        // DUP PATH
                        //#[cfg(target_os = "solana")]
                        core::arch::asm! {
                            // Load index byte
                            "ldxb r3, [r1 + 0]",
                            // Calculate index, load account into r3
                            "mul64 r3, 8",
                            "add64 r3, r4",
                            "ldxdw r3, [r3 + 0]",
                            // Store in r7 and advance input cursor
                            "stxdw [r7 + 0], r3",
                            "add64 r1, 8",
                            // Move to next account slot
                            "add64 r7, 8",

                            options(nostack),
                        };

                        // INLINED TAIL CALL - full remaining tree
                        #next_tree
                    }
                }
            };

            // NONDUP PATH
            //#[cfg(target_os = "solana")]
            core::arch::asm! {
                // Store account ptr and load account data len
                "stxdw [r7 + 0], r1",
                "ldxdw r8, [r1 + 80 + 8]",
                // Advance input cursor by static data, account data, and round up to next 8
                "add64 r1, {account_total}",
                "add64 r1, r8",
                "and64 r1, 0xFFFFFFFFFFFFFFF8",
                // Move to next account slot
                "add64 r7, 8",

                account_total = const TOTAL_ACCOUNT_DATA_TO_SKIP,
                options(nostack),
            };

            // INLINED TAIL CALL - full remaining tree
            #next_tree
        }
    }
}
