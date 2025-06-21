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
}

impl Parse for DispatchInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DispatchInput {
            value_expr: input.parse()?,
            _comma1: input.parse()?,
            max: input.parse()?,
            _comma2: input.parse()?,
            callee: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn binary_tree_dispatch(input: TokenStream) -> TokenStream {
    let DispatchInput {
        value_expr,
        _comma1: _,
        max,
        _comma2: _,
        callee,
    } = parse_macro_input!(input as DispatchInput);
    let max_val = max
        .base10_parse::<usize>()
        .expect("Expected integer literal");

    fn generate_tree(
        val_expr: &Expr,
        callee: &Ident,
        low: usize,
        high: usize,
    ) -> proc_macro2::TokenStream {
        let range_size = high - low + 1;

        match range_size {
            1 => {
                let call_lit = syn::LitInt::new(&low.to_string(), proc_macro2::Span::call_site());
                quote! {
                        #callee!(#call_lit)
                }
            }
            2 => {
                let high_lit = syn::LitInt::new(&high.to_string(), proc_macro2::Span::call_site());
                let low_lit = syn::LitInt::new(&low.to_string(), proc_macro2::Span::call_site());
                let high_call = quote! { #callee!(#high_lit) };
                let low_call = quote! { #callee!(#low_lit) };
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

                let high_call = quote! { #callee!(#high_lit) };
                let mid_call = quote! { #callee!(#mid_lit) };
                let low_call = quote! { #callee!(#low_lit) };

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
                let left_branch = generate_tree(val_expr, callee, low, mid);
                let right_branch = generate_tree(val_expr, callee, mid + 1, high);

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

    let tree = generate_tree(&value_expr, &callee, 0, max_val);
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

/// entrypoint_process_batched!(num_accounts, batch_size)
struct BatchedInput {
    total_expr: Expr,
    _comma: Token![,],
    batch_size: LitInt,
}

impl Parse for BatchedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(BatchedInput {
            total_expr: input.parse()?,
            _comma: input.parse()?,
            batch_size: input.parse()?,
        })
    }
}

/// Shared tree generator: `suppress_return = true` for batch-trees,
/// `false` for the final (remainder) tree that should `return`.
fn generate_account_tree(
    remaining: usize,
    total: usize,
    is_first_in_program: bool,
    suppress_return: bool,
) -> proc_macro2::TokenStream {
    // Base: no more accounts
    if remaining == 0 {
        if suppress_return {
            quote! { /* batch complete, continue */ }
        } else {
            quote! {
                // Batch complete
                return
            }
        }
    } else {
        let is_first_account = is_first_in_program && remaining == total;
        let next =
            generate_account_tree(remaining - 1, total, is_first_in_program, suppress_return);

        if is_first_account {
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
                #next
            }
        } else {
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

                            // Inlined tail for remaining accounts
                            #next
                        }
                    }
                };

                // Non-dup path
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

                // Inlined tail for remaining accounts
                #next
            }
        }
    }
}

#[proc_macro]
pub fn entrypoint_process_batched(input: TokenStream) -> TokenStream {
    let BatchedInput {
        total_expr,
        batch_size,
        ..
    } = parse_macro_input!(input as BatchedInput);

    // batch_size literal → usize
    let batch_val = batch_size
        .base10_parse::<usize>()
        .expect("batch_size must be integer literal");
    let max_lit = {
        let max = batch_val - 1;
        syn::LitInt::new(&max.to_string(), batch_size.span())
    };

    // Pre-generate batch trees without returns
    let first_tree = generate_account_tree(batch_val, batch_val, true, false);
    let loop_tree = generate_account_tree(batch_val, batch_val, false, false);

    let expanded = quote! {{
        let total = #total_expr;
        let mut num_full_batches_remaining = total / #batch_val -1;
        let remainder        = total % #batch_val;

        // First batch
        let a = { #[inline(always)] || { #first_tree } };
        a();


        // Remaining full batches
        while num_full_batches_remaining > 0{
            num_full_batches_remaining -= 1;
            let a = { #[inline(always)] || { #loop_tree } };
            a();
        }

        // Tail dispatch for remainder
        binary_tree_dispatch!(remainder, #max_lit, entrypoint_process_remainder);
    }};

    expanded.into()
}

#[proc_macro]
pub fn entrypoint_process_remainder(input: TokenStream) -> TokenStream {
    // remainder is always a literal here
    let total = parse_macro_input!(input as LitInt)
        .base10_parse::<usize>()
        .unwrap();

    // Generate a tree that WILL return at the end
    let tree = generate_account_tree(total, total, false, false);

    let result = quote! {{
        #tree
    }};
    result.into()
}
