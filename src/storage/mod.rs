mod memory;
mod sleddb;

pub use memory::MemoryDb;
pub use sleddb::SledDb;

use crate::{Result, pb::{Value, KvPair}};

// 由于后面要跨线程，需要添加该约束。(如果T实现了Send + Sync + 'static，则Arc<T>也实现了)
// 当我们使用具体类型时，如果该类型T实现了 Send + Sync + 'static，就可以不加
// 而使用泛型 Store: Storage 时，则没有该约束
pub trait Storage: Send + Sync + 'static {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>>;

    fn set(&self, table: &str, key: impl Into<String>, value: Value) -> Result<Option<Value>>;

    fn contains(&self, table: &str, key: &str) -> Result<bool>;

    fn delete(&self, table: &str, key: &str) -> Result<Option<Value>>;

    fn get_all(&self, table: &str) -> Result<Vec<KvPair>>;

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = KvPair>>>;
}

struct StorageItem<T> {
    iter: T
}

impl<T> StorageItem<T> {
    fn new(iter: T) -> Self {
        Self {
            iter
        }
    }
}

impl<T> Iterator for StorageItem<T> 
where
    T: Iterator,
    T::Item: Into<KvPair>
{
    type Item = KvPair;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.into())
    }
}