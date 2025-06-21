1) remove r9, r5 copy for num accounts
2) batched +7 into the {accounts_total} e.g. static accounts data
3) avoided a jump by reusing the nondup inline /* not relevant for optimal impl */
4) most optimal: do binary search for num accounts (worsens low N, significantly improves high N) and then use inlined unrolled loop with exactly that many num accounts (saves O(2*n) add64/jeq check for O(log2(N))) search (and avoid subtraction)


5) "balanced optimal": 

if accounts_remaining > 8 {
    process_eight();
    add64 r5, 8 // i.e. batched 8 adds
} else {
    for 0..accounts_remaining {
        // process
    }
}