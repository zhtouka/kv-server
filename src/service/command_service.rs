use crate::pb::{Hget, Hset, Hmget, Hmset, Value, Hexists, Hmexists, Hdelete, Hmdelete, Hgetall};
use crate::{storage::Storage, pb::CommandResponse};
use crate::{KvError};

pub trait CommandService {
    fn execute(self, store: &impl Storage) -> CommandResponse;
}

impl CommandService for Hget {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hget { table, key } = self;

        match store.get(&table, &key) {
            Ok(Some(value)) => value.into(),
            Ok(None) => KvError::NotFound(table, key).into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hmget {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hmget { table, keys } = self;

        keys
            .into_iter()
            .map(|key| match store.get(&table, &key) {
                Ok(Some(value)) => value,
                _ => Value { value: None },
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl CommandService for Hset {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hset { table, pair } = self;

        let (key, value) = pair
            .map(|x| (x.key, x.value.unwrap()))
            .unwrap();

        match store.set(&table, key, value) {
            Ok(Some(value)) => value.into(),
            Ok(None) => Value { value: None }.into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hmset {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hmset { table, pairs } = self;

        let values = pairs
            .into_iter()
            .map(|pair| {
                let (key, value) = (pair.key, pair.value.unwrap());
                match store.set(&table, key, value) {
                    Ok(Some(value)) => value,
                    _ => Value { value: None },
                }
            })
            .collect::<Vec<_>>();

        values.into()
    }
}

impl CommandService for Hexists {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hexists { table, key } = self;

        let value: Value = match store.contains(&table, &key) {
            Ok(b) => b.into(),
            Err(_) => false.into(),
        };
        value.into()
    }
}

impl CommandService for Hmexists {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hmexists { table, keys } = self;
        
        let values = keys
            .into_iter()
            .map(|key| match store.contains(&table, &key) {
                Ok(b) => b.into(),
                Err(_) => false.into(),
            })
            .collect::<Vec<Value>>();
        
        values.into()
    }
}

impl CommandService for Hdelete {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hdelete { table, key } = self;

        match store.delete(&table, &key) {
            Ok(Some(value)) => value.into(),
            Ok(None) => Value { value: None }.into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hmdelete {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let Hmdelete { table, keys } = self;

        let values = keys
            .into_iter()
            .map(|key| match store.delete(&table, &key) {
                Ok(Some(value)) => value,
                _ => Value { value: None },
            })
            .collect::<Vec<_>>();

        values.into()
    }
}

impl CommandService for Hgetall {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        let table = self.table;
        match store.get_all(&table) {
            Ok(values) => values.into(),
            Err(e) => e.into(),
        }
    }
}