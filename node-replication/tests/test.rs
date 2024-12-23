#[cfg(test)]
mod tests {
    use node_replication::nr::{ReplicaToken, ThreadToken, Log};
    use node_replication::nr::log::DEFAULT_LOG_BYTES;
    use node_replication::replica::MAX_THREADS_PER_REPLICA;
    use node_replication::context::{Context, MAX_PENDING_OPS};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    #[derive(Clone, Debug, PartialEq)]
    enum TestOp {
        Write(u64),
        Read,
        Delete
    }

    // Basic token and registration tests
    
    #[test]
    fn test_basic_token() {
        let thread_id = 1;
        let replica_token = unsafe { ReplicaToken::new(thread_id) };
        assert_eq!(replica_token.tid(), thread_id);
        
        let thread_id = MAX_THREADS_PER_REPLICA / 2;  // Test mid-range value
        let replica_token = unsafe { ReplicaToken::new(thread_id) };
        assert_eq!(replica_token.tid(), thread_id);
    }

    #[test]
    fn test_log_basic_registration() {
        let log = Log::<TestOp>::new_with_bytes(DEFAULT_LOG_BYTES, ());
        
        // Test multiple registrations work
        for _ in 0..4 {
            let token = log.register();
            assert!(token.is_some());
        }
    }

    // Context operation tests

    #[test]
    fn test_context_basic_ops() {
        let context = Context::<TestOp, Result<u64, ()>, ()>::new(1);
        
        // Basic enqueue and response
        assert!(context.enqueue(TestOp::Write(42), ()));
        context.enqueue_resp(Ok(42));
        assert_eq!(context.res(), Some(Ok(42)));

        // Multiple operations
        assert!(context.enqueue(TestOp::Read, ()));
        assert!(context.enqueue(TestOp::Write(43), ()));
        context.enqueue_resp(Ok(0));  // Read response
        context.enqueue_resp(Ok(43)); // Write response
        assert_eq!(context.res(), Some(Ok(0)));
        assert_eq!(context.res(), Some(Ok(43)));
    }

    #[test]
    fn test_context_capacity() {
        let context = Context::<TestOp, Result<u64, ()>, ()>::new(1);
        
        // Fill to capacity
        for i in 0..MAX_PENDING_OPS {
            assert!(context.enqueue(TestOp::Write(i as u64), ()));
        }
        
        // Verify overflow behavior
        assert!(!context.enqueue(TestOp::Write(42), ()));
    }

    // Concurrent operation tests

    #[test]
    fn test_concurrent_basic_registration() {
        let log = Arc::new(Log::<TestOp>::new_with_bytes(DEFAULT_LOG_BYTES, ()));
        let success_count = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        
        // Spawn multiple threads trying to register
        for _ in 0..4 {
            let log = Arc::clone(&log);
            let success_count = Arc::clone(&success_count);
            
            let handle = thread::spawn(move || {
                if let Some(_token) = log.register() {
                    success_count.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify some registrations succeeded
        assert!(success_count.load(Ordering::SeqCst) > 0);
    }

    // Response ordering tests

    #[test]
    fn test_response_ordering_single() {
        let context = Context::<TestOp, Result<u64, ()>, ()>::new(1);
        let test_values = vec![1, 2, 3, 4];
        
        // Enqueue operations in order
        for val in &test_values {
            assert!(context.enqueue(TestOp::Write(*val), ()));
            context.enqueue_resp(Ok(*val));
        }
        
        // Verify responses come back in order
        for val in &test_values {
            assert_eq!(context.res(), Some(Ok(*val)));
        }
        assert_eq!(context.res(), None);
    }

    #[test]
    fn test_batch_response_ordering() {
        let context = Context::<TestOp, Result<u64, ()>, ()>::new(1);
        let responses = vec![Ok(1), Ok(2), Ok(3)];
        
        // Enqueue operations
        for i in 1..=3 {
            assert!(context.enqueue(TestOp::Write(i), ()));
        }
        
        // Batch response enqueue
        context.enqueue_resps(&responses);
        
        // Verify order preservation
        for expected in responses {
            assert_eq!(context.res(), Some(expected));
        }
        assert_eq!(context.res(), None);
    }

    // Constants and limits tests

    #[test]
    fn test_constants_validity() {
        // Verify power of two requirements
        assert!(MAX_THREADS_PER_REPLICA.is_power_of_two(), 
            "MAX_THREADS_PER_REPLICA must be a power of two");
        assert!(MAX_PENDING_OPS.is_power_of_two(), 
            "MAX_PENDING_OPS must be a power of two");
        
        // Verify context batch size
        assert_eq!(Context::<TestOp, Result<u64, ()>, ()>::batch_size(), MAX_PENDING_OPS);
    }
}