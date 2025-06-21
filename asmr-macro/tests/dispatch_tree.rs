use asmr_macro::binary_tree_dispatch;

// Mock for values below transition (fully inlined version)
macro_rules! mock_process_inline {
    ($n:literal) => {{
        println!("dispatched to inline version: {}", $n);
        $n
    }};
}

// Mock for values at or above transition (batched version)
macro_rules! mock_process_batched {
    ($n:literal) => {{
        println!("dispatched to batched version: {}", $n);
        $n + 1000 // Add 1000 to distinguish from inline version in tests
    }};
}

// Test with transition at 12 (common case: <=11 use inline, >=12 use batched)
fn test_dispatch_with_transition(val: usize) -> usize {
    binary_tree_dispatch!(val, 20, mock_process_inline, 12, mock_process_batched)
}

// Test with transition at 5
fn test_dispatch_transition_5(val: usize) -> usize {
    binary_tree_dispatch!(val, 10, mock_process_inline, 5, mock_process_batched)
}

#[test]
fn test_binary_tree_dispatch_transition_at_12() {
    // Values below 12 should use inline version
    assert_eq!(test_dispatch_with_transition(0), 0);
    assert_eq!(test_dispatch_with_transition(1), 1);
    assert_eq!(test_dispatch_with_transition(5), 5);
    assert_eq!(test_dispatch_with_transition(10), 10);
    assert_eq!(test_dispatch_with_transition(11), 11);

    // Values at or above 12 should use batched version (returns value + 1000)
    assert_eq!(test_dispatch_with_transition(12), 1012);
    assert_eq!(test_dispatch_with_transition(13), 1013);
    assert_eq!(test_dispatch_with_transition(15), 1015);
    assert_eq!(test_dispatch_with_transition(20), 1020);
}

#[test]
fn test_binary_tree_dispatch_transition_at_5() {
    // Values below 5 should use inline version
    assert_eq!(test_dispatch_transition_5(0), 0);
    assert_eq!(test_dispatch_transition_5(1), 1);
    assert_eq!(test_dispatch_transition_5(2), 2);
    assert_eq!(test_dispatch_transition_5(3), 3);
    assert_eq!(test_dispatch_transition_5(4), 4);

    // Values at or above 5 should use batched version (returns value + 1000)
    assert_eq!(test_dispatch_transition_5(5), 1005);
    assert_eq!(test_dispatch_transition_5(6), 1006);
    assert_eq!(test_dispatch_transition_5(7), 1007);
    assert_eq!(test_dispatch_transition_5(8), 1008);
    assert_eq!(test_dispatch_transition_5(9), 1009);
    assert_eq!(test_dispatch_transition_5(10), 1010);
}

#[test]
fn test_edge_cases() {
    // Test transition at 0 (all values use batched)
    fn all_batched(val: usize) -> usize {
        binary_tree_dispatch!(val, 5, mock_process_inline, 0, mock_process_batched)
    }
    assert_eq!(all_batched(0), 1000);
    assert_eq!(all_batched(1), 1001);
    assert_eq!(all_batched(5), 1005);

    // Test transition at max+1 (all values use inline)
    fn all_inline(val: usize) -> usize {
        binary_tree_dispatch!(val, 5, mock_process_inline, 6, mock_process_batched)
    }
    assert_eq!(all_inline(0), 0);
    assert_eq!(all_inline(3), 3);
    assert_eq!(all_inline(5), 5);
}

// Real-world example showing how this would be used with entrypoint macros
#[test]
fn test_entrypoint_dispatch_pattern() {
    macro_rules! entrypoint_process {
        ($n:literal) => {{
            println!("Using fully inlined entrypoint for {} accounts", $n);
            // Would generate 2^n paths
            $n * 2
        }};
    }

    macro_rules! entrypoint_process_batched {
        ($n:literal) => {{
            println!("Using batched entrypoint for {} accounts", $n);
            // Would generate multiple 2^8 path batches
            $n * 3
        }};
    }

    fn dispatch_entrypoint(num_accounts: usize) -> usize {
        // Use inline for <=11, batched for >=12
        binary_tree_dispatch!(
            num_accounts,
            100,
            entrypoint_process,
            12,
            entrypoint_process_batched
        )
    }

    // Small account counts use fully inlined (returns n*2)
    assert_eq!(dispatch_entrypoint(5), 10);
    assert_eq!(dispatch_entrypoint(11), 22);

    // Large account counts use batched (returns n*3)
    assert_eq!(dispatch_entrypoint(12), 36);
    assert_eq!(dispatch_entrypoint(25), 75);
    assert_eq!(dispatch_entrypoint(100), 300);
}
