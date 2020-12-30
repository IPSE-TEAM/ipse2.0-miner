use std::fmt;
use std::io::{self, Result};
use kvdb_rocksdb::{DatabaseConfig, Database};
use kvdb::KeyValueDB;


/// Required length of prefixes.
/// key-value-timestamp-is_expire-order_id
pub const META_COL: u32 = 0;

pub struct KVDatabase {
    pub(crate) config: DatabaseConfig,
    pub(crate) path: String,
}


impl KVDatabase {
    /// create new database for rockets db
    pub fn new(config: DatabaseConfig, path: String) -> KVDatabase {
        KVDatabase {
            config,
            path,
        }
    }

    pub fn client(&self) -> Result<Database> {
        Database::open(&self.config, &*self.path)
    }

    // /// put a key-value into db
    // pub fn put(self, key: &[u8], value: &[u8]) -> io::Result<()> {
    //     let mut transaction = self.db.unwrap().transaction();
    //     transaction.put(META_COL, key, value);
    //     self.db.unwrap().write(transaction)?;
    //     Ok(())
    // }
    // /// Get a value by key.
    // pub fn get(self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
    //     self.db.unwrap().get(META_COL, key)
    // }
    //
    // pub fn delete(self, key: &[u8]) -> io::Result<()> {
    //     let mut transaction = self.db.unwrap().transaction();
    //     transaction.delete(META_COL, key);
    //     self.db.unwrap().write(transaction)?;
    //     Ok(())
    // }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::{self};
    use tempdir::TempDir;

    fn init_db(columns: u32) -> io::Result<Database> {
        let tempdir = TempDir::new("")?;
        let config = DatabaseConfig::with_columns(columns);
        KVDatabase::new(config, tempdir.path().to_str().expect("tempdir path is valid unicode").to_string()).client()
    }

    fn init_kv_data(columns: u32) -> io::Result<KVDatabase> {
        let tempdir = TempDir::new("")?;
        let config = DatabaseConfig::with_columns(columns);
        Ok(KVDatabase {
            config,
            path: tempdir.path().to_str().expect("tempdir path is valid unicode").parse().unwrap(),
        })
    }


    #[test]
    fn put_and_get() -> io::Result<()> {
        let client = init_db(1)?;
        let key1 = b"key1";

        let mut transaction = client.transaction();
        transaction.put(0, key1, b"value1");
        client.write(transaction)?;
        assert_eq!(&*client.get(0, key1)?.unwrap(), b"value1");
        Ok(())
    }

    #[test]
    fn kv_data_put_and_delete() -> io::Result<()> {
        let client = init_kv_data(1).unwrap().client()?;
        let key1 = b"key1";
        let value1 = b"value1";

        let mut transaction = client.transaction();
        transaction.put(0, key1, b"value1");
        client.write(transaction)?;
        assert_eq!(&*client.get(0, key1)?.unwrap(), b"value1");
        Ok(())
    }
}



