use node_replication::{Dispatch, Log, LogMapper, Replica};
use std::sync::Arc;

// Test structure implementing Dispatch
#[derive(Debug, Default)]
struct TestMap {
    storage: std::collections::HashMap<u64, u64>
}

#[derive(Debug, Clone)]
enum Op {
    Put(u64, u64),
    Remove(u64)
}

impl Dispatch for TestMap {
    type ReadOperation<'rop> = u64;
    type WriteOperation = Op;
    type Response = Option<u64>;

    fn dispatch<'rop>(&self, op: Self::ReadOperation<'rop>) -> Self::Response {
        self.storage.get(&op).copied()
    }

    fn dispatch_mut(&mut self, op: Self::WriteOperation) -> Self::Response {
        match op {
            Op::Put(k, v) => self.storage.insert(k, v),
            Op::Remove(k) => self.storage.remove(&k),
        }
    }
}

#[kani::proof]
fn verify_replica_consistency() {
    let ds = Arc::new(TestMap::default());
    let log = Arc::new(Log::new(LogMapper::new(ds.clone())));
    let replica = Replica::new(log);

    // Test write operation
    let key = kani::any();
    let value = kani::any();
    let write_op = Op::Put(key, value);
    
    replica.execute_mut(write_op);
    
    // Verify read reflects write
    let read_result = replica.execute(key);
    assert!(read_result == Some(value), "Write not visible in read");
}

#[kani::proof]
fn verify_operation_atomicity() {
    let ds = Arc::new(TestMap::default());
    let log = Arc::new(Log::new(LogMapper::new(ds.clone())));
    let replica = Replica::new(log);

    let op1 = Op::Put(kani::any(), kani::any());
    let op2 = Op::Put(kani::any(), kani::any());

    replica.execute_mut(op1.clone());
    replica.execute_mut(op2.clone());
    
    // Verify operations are atomic
    match (op1, op2) {
        (Op::Put(k1, v1), Op::Put(k2, v2)) => {
            if k1 != k2 {
                assert!(replica.execute(k1) == Some(v1));
                assert!(replica.execute(k2) == Some(v2));
            }
        },
        _ => ()
    }
}