use sled::Db;
use sled::IVec;

use crate::KvPair;
use crate::Value;
use crate::Result;
use crate::storage::StorageItem;

use super::Storage;

#[derive(Debug, Clone)]
pub struct SledDb (Db);

impl SledDb {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self(sled::open(path).expect("failed to create db"))
    }

    fn get_full_name(&self, table: &str, key: &str) -> String {
        format!("{}:{}", table, key)
    }

    fn get_prefix(&self, table: &str) -> String {
        format!("{}", table)
    }
}

impl Storage for SledDb {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>> {
        let name = self.get_full_name(table, key);

        self.0.get(name)?
            .map(|x| x.as_ref().try_into())
            .transpose()
    }

    fn set(&self, table: &str, key: impl Into<String>, value: Value) -> Result<Option<Value>> {
        let name = self.get_full_name(table, &key.into());

        let value: Vec<u8> = value.try_into()?;

        self.0.insert(name, value)?
            .map(|x| x.as_ref().try_into())
            .transpose()
    }

    fn contains(&self, table: &str, key: &str) -> Result<bool> {
        let name = self.get_full_name(table, key);
        
        Ok(self.0.contains_key(&name)?)
    }

    fn delete(&self, table: &str, key: &str) -> Result<Option<Value>> {
        let name = self.get_full_name(table, key);

        self.0.remove(name)?
            .map(|x| x.as_ref().try_into())
            .transpose()
    }

    fn get_all(&self, table: &str) -> Result<Vec<KvPair>> {
        let prefix = self.get_prefix(table);

        let value = self.0.scan_prefix(prefix)
            .map(|x| x.into())
            .collect::<Vec<_>>();
        
        Ok(value)
    }

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = KvPair>>> {
        let prefix = self.get_prefix(table);
        
        let value = self.0.scan_prefix(prefix);

        Ok(Box::new(StorageItem::new(value)))
    }
}


impl From<sled::Result<(IVec, IVec)>> for KvPair {
    fn from(value: sled::Result<(IVec, IVec)>) -> Self {
        match value {
            Ok((k, v)) => match v.as_ref().try_into() {
                Ok(value) => (ivec_to_key(&k), value).into(),
                Err(_) => KvPair::default(),
            },
            _ => KvPair::default(),
        }
    }
}

fn ivec_to_key(ivec: &[u8]) -> &str {
    let str = std::str::from_utf8(ivec).unwrap();
    let mut s = str.split(":");
    s.next();
    s.next().unwrap()
}


#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::storage::Storage;

    use super::SledDb;


    #[test]
    fn sled_db_should_work() {
        let dir = tempdir().unwrap();
        
        let db = SledDb::new(dir.path());

        let res = db.set("t1", "k1", "v1".into()).unwrap();
        assert!(res.is_none());

        let res = db.get("t1", "k1").unwrap();
        assert_eq!(res, Some("v1".into()));

        let res = db.contains("t1", "k1").unwrap();
        assert!(res);

        db.set("t1", "k2", "v2".into()).unwrap();
        let mut res = db.get_all("t1").unwrap();
        res.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(res, vec![("k1", "v1".into()).into(), ("k2", "v2".into()).into()]);

        let mut res = db.get_iter("t1").unwrap().collect::<Vec<_>>();
        res.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(res, vec![("k1", "v1".into()).into(), ("k2", "v2".into()).into()]);

        let res = db.delete("t1", "k1").unwrap();
        assert_eq!(res, Some("v1".into()));
        let res = db.delete("t1", "k2").unwrap();
        assert_eq!(res, Some("v2".into()));
        dir.close().unwrap();
    }
}