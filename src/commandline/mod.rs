mod command;

pub use command::*;

use crate::{CommandRequest, command_request::RequestData};

use self::command::{Get, Set, Exists, Delete, Subscribe, Unsubscribe, Publish};

impl From<Get> for CommandRequest {
    fn from(value: Get) -> Self {
        Self {
            request_data: Some(RequestData::Hget(crate::Hget {
                table: value.table,
                key: value.key,
            })),
        }
    }
}

impl From<Set> for CommandRequest {
    fn from(value: Set) -> Self {
        Self {
            request_data: Some(RequestData::Hset(crate::Hset {
                table: value.table,
                pair: Some((value.key, value.value.into()).into()),
            })),
        }
    }
}

impl From<Exists> for CommandRequest {
    fn from(value: Exists) -> Self {
        Self {
            request_data: Some(RequestData::Hexists(crate::Hexists { 
                table: value.table, 
                key: value.key 
            })),
        }
    }
}

impl From<Delete> for CommandRequest {
    fn from(value: Delete) -> Self {
        Self {
            request_data: Some(RequestData::Hdelete(crate::Hdelete {
                table: value.table, 
                key: value.key 
            })),
        }
    }
}

impl From<Subscribe> for CommandRequest {
    fn from(value: Subscribe) -> Self {
        Self {
            request_data: Some(RequestData::Subscribe(crate::Subscribe {
                topic: value.topic,
            })),
        }
    }
}

impl From<Unsubscribe> for CommandRequest {
    fn from(value: Unsubscribe) -> Self {
        Self {
            request_data: Some(RequestData::Unsubscribe(crate::Unsubscribe {
                topic: value.topic,
                id: value.id
            })),
        }
    }
}

impl From<Publish> for CommandRequest {
    fn from(value: Publish) -> Self {
        Self {
            request_data: Some(RequestData::Publish(crate::Publish {
                topic: value.topic,
                data: value.data.into_iter().map(|x| x.into()).collect::<Vec<_>>()
            })),
        }
    }
}

impl From<MGet> for CommandRequest {
    fn from(value: MGet) -> Self {
        Self {
            request_data: Some(RequestData::Hmget(crate::Hmget {
                table: value.table,
                keys: value.keys,
            })),
        }
    }
}

impl From<MExists> for CommandRequest {
    fn from(value: MExists) -> Self {
        Self {
            request_data: Some(RequestData::Hmexists(crate::Hmexists {
                table: value.table,
                keys: value.keys,
            })),
        }
    }
}

impl From<MDelete> for CommandRequest {
    fn from(value: MDelete) -> Self {
        Self {
            request_data: Some(RequestData::Hmdelete(crate::Hmdelete {
                table: value.table,
                keys: value.keys,
            })),
        }
    }
}

impl From<GetAll> for CommandRequest {
    fn from(value: GetAll) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(crate::Hgetall {
                table: value.table,
            })),
        }
    }
}

impl From<self::command::Value> for crate::Value {
    fn from(value: self::command::Value) -> Self {
        match value {
            self::command::Value::Text { text } => text.into(),
            self::command::Value::Integer { integer } => integer.into(),
            self::command::Value::Double { double } => double.into(),
            self::command::Value::Boolean { boolean } => boolean.into(),
            self::command::Value::Binary { binary } => binary.into(),
        }
    }
}