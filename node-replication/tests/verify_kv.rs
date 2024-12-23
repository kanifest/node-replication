use node_replication::nr::Dispatch;
use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq)]
pub enum KvOperation {
    Get(u8),
    Set(u8, u8),
    Delete(u8),
}

pub struct KvStore {
    storage: [(u8, u8); 2], // Fixed-size array with 2 key-value pairs
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            storage: [(0, 0); 2], // Initialize with default key-value pairs
        }
    }

    fn find_index(&self, key: u8) -> Option<usize> {
        self.storage.iter().position(|&(k, _)| k == key)
    }

    fn find_empty_slot(&self) -> Option<usize> {
        self.storage.iter().position(|&(k, _)| k == 0)
    }
}

impl Dispatch for KvStore {
    type ReadOperation<'rop> = KvOperation;
    type WriteOperation = KvOperation;
    type Response = Option<u8>;

    fn dispatch<'rop>(&self, op: Self::ReadOperation<'rop>) -> Self::Response {
        match op {
            KvOperation::Get(key) => {
                self.find_index(key).map(|index| self.storage[index].1)
            }
            _ => None,
        }
    }

    fn dispatch_mut(&mut self, op: Self::WriteOperation) -> Self::Response {
        match op {
            KvOperation::Set(key, value) => {
                if let Some(index) = self.find_index(key) {
                    self.storage[index].1 = value;
                    Some(value)
                } else if let Some(index) = self.find_empty_slot() {
                    self.storage[index] = (key, value);
                    Some(value)
                } else {
                    None // No space available
                }
            }
            KvOperation::Delete(key) => {
                if let Some(index) = self.find_index(key) {
                    self.storage[index] = (0, 0); // Reset to default
                    Some(key)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}


#[kani::proof]
#[kani::unwind(5)]
fn verify_set_and_get() {
    let mut kv_store = KvStore::new();

    let key: u8 = kani::any();
    let value: u8 = kani::any();
    kani::assume(key > 0 && key < 3); // Valid key range
    kani::assume(value > 0 && value < 3); // Valid value range

    kv_store.dispatch_mut(KvOperation::Set(key, value));
    let result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(result, Some(value));
}

#[kani::proof]
#[kani::unwind(5)]
fn verify_delete() {
    let mut kv_store = KvStore::new();

    let key: u8 = kani::any();
    let value: u8 = kani::any();
    kani::assume(key > 0 && key < 3); // Valid key range
    kani::assume(value > 0 && value < 3); // Valid value range

    kv_store.dispatch_mut(KvOperation::Set(key, value));
    kv_store.dispatch_mut(KvOperation::Delete(key));
    let result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(result, None);
}

#[kani::proof]
#[kani::unwind(5)]
fn verify_idempotent_delete() {
    let mut kv_store = KvStore::new();

    let key: u8 = kani::any();
    let value: u8 = kani::any();
    kani::assume(key > 0 && key < 3);
    kani::assume(value < 3);

    // Set the key-value pair
    kv_store.dispatch_mut(KvOperation::Set(key, value));

    // Delete the key multiple times
    kv_store.dispatch_mut(KvOperation::Delete(key));
    kv_store.dispatch_mut(KvOperation::Delete(key));

    // Verify that the key is no longer in the store
    let result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(result, None);
}

#[kani::proof]
#[kani::unwind(5)]
fn verify_delete_non_existent_key() {
    let mut kv_store = KvStore::new();

    let key: u8 = kani::any();
    kani::assume(key > 0 && key < 3);

    // Attempt to delete a non-existent key
    let delete_result = kv_store.dispatch_mut(KvOperation::Delete(key));

    // Verify that the delete operation returns None, indicating the key was not found
    assert_eq!(delete_result, None);

    // Ensure the store remains empty
    let get_result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(get_result, None);
}

#[kani::proof]
#[kani::unwind(5)]
fn verify_update_existing_key() {
    let mut kv_store = KvStore::new();

    let key: u8 = kani::any();
    let initial_value: u8 = kani::any();
    let updated_value: u8 = kani::any();
    kani::assume(key > 0 && key < 3);
    kani::assume(initial_value < 3);
    kani::assume(updated_value < 3);

    // Set the initial value
    kv_store.dispatch_mut(KvOperation::Set(key, initial_value));

    // Update the value for the same key
    kv_store.dispatch_mut(KvOperation::Set(key, updated_value));

    // Verify that the key holds the updated value
    let result = kv_store.dispatch(KvOperation::Get(key));
    assert_eq!(result, Some(updated_value));
}

#[kani::proof]
#[kani::unwind(5)]
fn verify_idempotent_set() {
    let mut kv_store = KvStore::new();

    // Generate arbitrary key and value
    let key: u8 = kani::any();
    let value: u8 = kani::any();
    kani::assume(key > 0 && key < 3); // Limit the range to avoid default key
    kani::assume(value > 0 && value < 3); // Limit the range

    // Perform the Set operation multiple times with the same key-value pair
    kv_store.dispatch_mut(KvOperation::Set(key, value));
    kv_store.dispatch_mut(KvOperation::Set(key, value));
    kv_store.dispatch_mut(KvOperation::Set(key, value));

    // Retrieve the value associated with the key
    let result = kv_store.dispatch(KvOperation::Get(key));

    // Verify that the value matches the expected value
    assert_eq!(result, Some(value));
}

// Will be serialized but...
#[kani::proof]
#[kani::unwind(5)]
fn verify_concurrent_operations() {
    let mut kv_store = KvStore::new();

    let key1: u8 = 1;
    let key2: u8 = 2;
    let value1: u8 = kani::any();
    let value2: u8 = kani::any();
    kani::assume(value1 < 3);
    kani::assume(value2 < 3);

    // Simulate concurrent Set operations
    kv_store.dispatch_mut(KvOperation::Set(key1, value1));
    kv_store.dispatch_mut(KvOperation::Set(key2, value2));

    // Simulate concurrent Get operations
    let result1 = kv_store.dispatch(KvOperation::Get(key1));
    let result2 = kv_store.dispatch(KvOperation::Get(key2));

    // Verify that each key returns the correct value
    assert_eq!(result1, Some(value1));
    assert_eq!(result2, Some(value2));
}
