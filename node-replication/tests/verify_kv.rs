use std::collections::HashMap;
use node_replication::nr::Dispatch;

#[derive(Debug, Clone, PartialEq)] // Derive PartialEq here
pub enum KvOperation {
    Get(u8),
    Set(u8, u8),
    Delete(u8),
}

pub struct KvStore {
    storage: HashMap<u8, u8>,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            storage: HashMap::new(),
        }
    }
}

impl Dispatch for KvStore {
    type ReadOperation<'rop> = KvOperation;
    type WriteOperation = KvOperation;
    type Response = Option<u8>;

    fn dispatch<'rop>(&self, op: Self::ReadOperation<'rop>) -> Self::Response {
        match op {
            KvOperation::Get(key) => self.storage.get(&key).cloned(),
            _ => None, // Handle other cases if necessary
        }
    }

    fn dispatch_mut(&mut self, op: Self::WriteOperation) -> Self::Response {
        match op {
            KvOperation::Set(key, value) => self.storage.insert(key, value),
            KvOperation::Delete(key) => self.storage.remove(&key),
            _ => None, // Handle other cases if necessary
        }
    }
}


#[kani::proof]
#[kani::unwind(5)]
fn verify_set_and_get() {
    let mut kv_store = KvStore::new();

    // Generate arbitrary key and value
    let key: u8 = kani::any();
    let value: u8 = kani::any();
    kani::assume(key < 2); // Limit the range
    kani::assume(value < 2); // Limit the range

    // Perform Set operation
    kv_store.dispatch_mut(KvOperation::Set(key.clone(), value.clone()));

    // Perform Get operation and verify the result
    let result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(result, Some(value));
}


use kani::any;

#[kani::proof]
#[kani::unwind(5)]
fn verify_delete() {
    let mut kv_store = KvStore::new();

    // Generate arbitrary key and value
    let key: u8 = kani::any();
    let value: u8 = kani::any();
    kani::assume(key < 2); // Limit the range
    kani::assume(value < 2); // Limit the range

    // Set the key-value pair
    kv_store.dispatch_mut(KvOperation::Set(key.clone(), value));

    // Delete the key
    kv_store.dispatch_mut(KvOperation::Delete(key.clone()));

    // Attempt to get the deleted key and verify it's None
    let result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(result, None);
}


#[kani::proof]
#[kani::unwind(5)]
fn verify_idempotent_set() {
    let mut kv_store = KvStore::new();

    // Generate arbitrary key and two different values
    let key: u8 = 0;
    let value1: u8 = kani::any();
    let value2: u8 = kani::any();
    kani::assume(value1 < 2); // Limit the range
    kani::assume(value2 < 2); // Limit the range

    // Set the key to value1
    kv_store.dispatch_mut(KvOperation::Set(key.clone(), value1));

    // Set the key to value2
    kv_store.dispatch_mut(KvOperation::Set(key.clone(), value2));

    // Get the key and verify it returns value2
    let result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(result, Some(value2));
}
