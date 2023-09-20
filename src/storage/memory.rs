use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;

use crate::pb::{Value, KvPair};
use crate::{Result, KvError};
use super::{Storage, StorageItem};



#[derive(Clone)]
pub struct MemoryDb {
    table: Arc<DashMap<String, DashMap<String, Value>>>
}

impl MemoryDb {
    pub fn new() -> Self {
        Self {
            table: Arc::new(DashMap::default()),
        }
    }

    pub fn get_or_create_table(&self, table: impl Into<String>) -> Ref<String, DashMap<String, Value>> {
        self.table.entry(table.into()).or_insert(DashMap::new()).downgrade()
    }
}

impl Storage for MemoryDb {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>> {
        match self.table.get(table) {
            Some(t) => {
                Ok(t.get(key).map(|x| x.value().to_owned()))
            },
            None => Err(KvError::NotFound(table.into(), key.into()))
        }
    }

    fn set(&self, table: &str, key: impl Into<String>, value: Value) -> Result<Option<Value>> {
        Ok(self.get_or_create_table(table)
            .insert(key.into(), value))
    }

    fn contains(&self, table: &str, key: &str) -> Result<bool> {
        match self.table.get(table) {
            Some(t) => Ok(t.contains_key(key)),
            None => Err(KvError::NotFound(table.into(), key.into()))
        }
    }

    fn delete(&self, table: &str, key: &str) -> Result<Option<Value>> {
        match self.table.get(table) {
            Some(t) => Ok(t.remove(key).map(|x| x.1)),
            None => Err(KvError::NotFound(table.into(), key.into()))
        }
    }

    fn get_all(&self, table: &str) -> Result<Vec<KvPair>> {
        match self.table.get(table) {
            Some(t) => {
                Ok(t
                    .iter()
                    .map(|x| (x.key().clone(), x.value().clone()).into())
                    .collect())
            },
            None => Err(KvError::NotFound(table.into(), "".into()))
        } 
    }

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = KvPair>>> {
        match self.table.get(table) {
            Some(x) => Ok(Box::new(StorageItem::new(x.clone().into_iter()))),
            None => Err(KvError::NotFound(table.into(), "".into()))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::{memory::MemoryDb, Storage};

    fn memory_db_init_and_set_initial_value() -> MemoryDb {
        let db = MemoryDb::new();
        db.set("t1", "k1", "v1".into()).unwrap();
        db
    }

    #[test]
    fn memory_db_set_should_work() {
        let db = MemoryDb::new();
        let a = db.set("t1", "k1", "v1".into()).unwrap();
        assert!(a.is_none());
    }

    #[test]
    fn memory_db_should_work() {
        let db = memory_db_init_and_set_initial_value();
        let a = db.get("t1", "k1").expect("db get error");
        assert_eq!(a, Some("v1".into()));

        let a = db.set("t1", "k1", "v2".into()).expect("db set error");
        assert_eq!(a, Some("v1".into()));

        let a = db.contains("t1", "k1").expect("db exists error");
        assert_eq!(a, true);

        db.set("t1", "k2", "v2".into()).expect("db set error");
        let mut v = db.get_all("t1").expect("db get all error");
        v.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(v, vec![("k1", "v2".into()).into(), ("k2", "v2".into()).into()]);

        let a = db.delete("t1", "k1").expect("db delete error");
        assert_eq!(a, Some("v2".into()));
    }

    #[test]
    fn memory_db_get_iter_should_work() {
        let db = memory_db_init_and_set_initial_value();
        db.set("t1", "k2", "v2".into()).expect("db set error");
        let a = db.get_iter("t1");
        assert!(a.is_ok());
        let mut a = a.unwrap();
        let mut v = vec![];
        while let Some(x) = a.next() {
            v.push(x.clone());
        }
        v.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(v, vec![("k1", "v1".into()).into(), ("k2", "v2".into()).into()]);
    }
}