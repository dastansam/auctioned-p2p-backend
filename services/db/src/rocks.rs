use std::{path::Iter, borrow::Cow, collections::hash_set};

use common_types::{IteratorMode, store::RecordStore, Key, Record, ProviderRecord, PeerId};
pub use rocksdb::{Options, WriteBatch, DB, DBIterator};
pub use common_types::{ Storage, Error, AppStorage};
use std::collections::{hash_map};
/// RocksDB instance
#[derive(Debug)]
pub struct RocksDB {
    pub db: DB,
}

impl RocksDB {
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<std::path::Path>,
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(RocksDB { db })
    }
}

/// Implement Storage trait for RocksDB
impl Storage for RocksDB {
    /// Write in the storage
    fn write<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        Ok(self.db.put(key, value)?)
    }

    /// Read from the storage
    fn read<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: AsRef<[u8]>,
    {
        Ok(self.db.get(key)?)
    }

    // Delete from the storage
    fn delete<K>(&self, key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        Ok(self.db.delete(key)?)
    }

    /// Check if the key exists in the storage
    fn contains<K>(&self, key: K) -> Result<bool, Error>
    where
        K: AsRef<[u8]>,
    {
        Ok(self.db.get(key)?.is_some())
    }

    fn iterator(&self, mode: IteratorMode) -> DBIterator
    {
        self.db.iterator(mode)
    }
}

impl AppStorage for RocksDB {}

// impl<'a> RecordStore<'a> for RocksDB {
//     type RecordsIter = std::iter::Map<
//         hash_map::Values<'a, Key, Record>,
//         fn(&'a Record) -> Cow<'a, Record>,
//     >;

//     type ProvidedIter = std::iter::Map<
//         hash_set::Iter<'a, ProviderRecord>,
//         fn(&'a ProviderRecord) -> Cow<'a, ProviderRecord>,
//     >;

//     fn get(&'a self, key: &Key) -> Option<Cow<'_, Record>> {
//         match self.db.get(key) {
//             Ok(Some(value)) => Some(Cow::Owned(Record::new(key.clone(), value))),
//             _ => None,
//         }
//     }

//     fn put(&'a mut self, record: Record) -> Result<(), common_types::store::Error> {
//         match self.db.put(record.key, record.value) {
//             Ok(_) => Ok(()),
//             // To-do: solve this issue, i.e converting error to common_types::store::Error
//             Err(e) => Err(common_types::store::Error::MaxRecords),
//         }
//     }

//     fn remove(&'a mut self, k: &Key){
//         match self.db.delete(k) {
//             Ok(_) => (),
//             // To-do: solve this issue, i.e converting error to common_types::store::Error
//             Err(e) => (),
//         }
//     }

//     fn add_provider(&'a mut self, record: ProviderRecord) -> common_types::store::Result<()> {
//         unimplemented!()
//     }

//     fn remove_provider(&'a mut self, k: &Key, provider: &PeerId){
//         unimplemented!()
//     }

//     fn providers(&'a self, key: &Key) -> Vec<ProviderRecord> {
//         unimplemented!()
//     }

//     fn records(&'a self) -> Self::RecordsIter {
//         let mut hashmap = hash_map::HashMap::new();    
//         self.iterator(IteratorMode::Start).map(|(k, v)| {
//             hashmap.insert(Key::new(&k), Record::new(Key::new(&k), v.into()));
//         });
//         hashmap.values().map(|record| Cow::Owned(record)).to_owned()
//     }

//     fn provided(&'a self) -> Self::ProvidedIter {
//         unimplemented!()
//     }
// }