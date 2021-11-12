mod rocks;
pub use memory::MemoryDB;

/// Interface for Key-Value storage
pub trait Storage {
    /// Get a value from the storage
    fn read<K>(&self, key: &K) -> Option<Vec<u8>>
    where
        K: AsRef<[u8]>;
    
    /// Put a value into the storage
    fn write<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;
    
    /// Delete a value from the storage
    fn delete<K>(&mut self, key: &K)
    where
        K: AsRef<[u8]>;

    /// Check if a key exists in the storage
    fn contains<K>(&self, key: &K) -> bool
    where
        K: AsRef<[u8]>;
    
    /// Get the number of keys in the storage
    // fn keys(&self) -> Vec<Vec<u8>>;
}