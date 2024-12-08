use node_replication::replica::{ReplicaToken, ThreadIdx, MAX_THREADS_PER_REPLICA};

// Verify ReplicaToken creation and constraints
#[kani::proof]
fn verify_replica_token_creation() {
    let thread_id: ThreadIdx = kani::any();
    
    // Verify only valid thread IDs can be used
    kani::assume(thread_id > 0 && thread_id <= MAX_THREADS_PER_REPLICA);
    
    // Create token
    let token = unsafe { ReplicaToken::new(thread_id) };
    
    // Verify properties
    assert_eq!(token.tid(), thread_id);
    assert!(token.tid() >= 1);
    assert!(token.tid() <= MAX_THREADS_PER_REPLICA);
}

// Verify thread ID constraints
#[kani::proof]
fn verify_thread_idx_constraints() {
    // Verify MAX_THREADS_PER_REPLICA is power of two
    assert!(MAX_THREADS_PER_REPLICA.is_power_of_two());
    
    // Verify thread ID bounds
    let thread_id: ThreadIdx = kani::any();
    if let Ok(token) = (|| -> Result<ReplicaToken, ()> {
        if thread_id > 0 && thread_id <= MAX_THREADS_PER_REPLICA {
            Ok(unsafe { ReplicaToken::new(thread_id) })
        } else {
            Err(())
        }
    })() {
        // Verify valid token properties
        assert!(token.tid() >= 1);
        assert!(token.tid() <= MAX_THREADS_PER_REPLICA);
    }
}

// Verify token uniqueness properties
#[kani::proof]
fn verify_token_uniqueness() {
    let id1: ThreadIdx = kani::any();
    let id2: ThreadIdx = kani::any();
    
    // Only test valid thread IDs
    kani::assume(id1 > 0 && id1 <= MAX_THREADS_PER_REPLICA);
    kani::assume(id2 > 0 && id2 <= MAX_THREADS_PER_REPLICA);
    
    let token1 = unsafe { ReplicaToken::new(id1) };
    let token2 = unsafe { ReplicaToken::new(id2) };
    
    if id1 == id2 {
        assert_eq!(token1.tid(), token2.tid());
    } else {
        assert_ne!(token1.tid(), token2.tid());
    }
}

// Verify token bounds
#[kani::proof]
fn verify_token_bounds() {
    let thread_id: ThreadIdx = kani::any();
    
    // Test both valid and invalid thread IDs
    if thread_id > 0 && thread_id <= MAX_THREADS_PER_REPLICA {
        let token = unsafe { ReplicaToken::new(thread_id) };
        // Valid token should maintain ID
        assert_eq!(token.tid(), thread_id);
    } else {
        // For invalid IDs, verify they're actually out of bounds
        assert!(thread_id == 0 || thread_id > MAX_THREADS_PER_REPLICA);
    }
}

// Verify token copy behavior
#[kani::proof]
fn verify_token_copy() {
    let thread_id: ThreadIdx = kani::any();
    kani::assume(thread_id > 0 && thread_id <= MAX_THREADS_PER_REPLICA);
    
    let token1 = unsafe { ReplicaToken::new(thread_id) };
    let token2 = token1; // Test Copy trait
    
    // Verify copy maintains identity
    assert_eq!(token1.tid(), token2.tid());
    assert_eq!(token2.tid(), thread_id);
}