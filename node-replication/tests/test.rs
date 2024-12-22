// Test cases
#[cfg(test)]

mod tests {
    use node_replication::nr::{ReplicaToken, ThreadToken};
    use node_replication::nr::Dispatch;
    use node_replication::context::Context;
    use node_replication::context::MAX_PENDING_OPS;
    use core::sync::atomic::Ordering;

    #[derive(Clone)]
    enum TestOp {
        Write(u64),
        Read
    }

    
    #[test]
    fn test_basic_token() {
        let thread_id = 1;
        let replica_token = unsafe { ReplicaToken::new(thread_id) };
        let _token = ThreadToken::new(thread_id, replica_token);
    }
}