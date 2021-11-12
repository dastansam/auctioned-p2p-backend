use super::{Storage};
pub use rocksdb::{Options, WriteBatch, DB};

/// RocksDB instance
#[derive(Debug)]
pub struct RocksDBInstance {
    pub db: DB,
}

/// Implement Storage trait for RocksDB
impl Storage for RocksDBInstance {
    /// Write in the storage
    fn write<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        Ok(self.db.put(key, value)?);
    }

    /// Read from the storage
    fn read<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: AsRef<[u8]>,
    {
        Ok(self.db.get(key)?)
    }

    // Delete from the storage
    fn delete<K>(&mut self, key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        Ok(self.db.delete(key)?);
    }

    /// Check if the key exists in the storage
    fn contains<K>(&self, key: K) -> Result<bool, Error>
    where
        K: AsRef<[u8]>,
    {
        Ok(self.db.get(key)?.is_some())
    }
}