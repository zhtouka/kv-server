mod abi;

pub use abi::*;
use http::StatusCode;
use prost::Message;

use crate::KvError;

use self::command_request::RequestData;

impl CommandRequest {
    pub fn new_hget(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hget(Hget {
                table: table.into(),
                key: key.into(),
            }))
        }
    }

    pub fn new_hmget(table: impl Into<String>, keys: Vec<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hmget(Hmget {
                table: table.into(),
                keys: keys,
            }))
        }
    }

    pub fn new_hset(table: impl Into<String>, key: impl Into<String>, value: Value) -> Self {
        Self {
            request_data: Some(RequestData::Hset(Hset {
                table: table.into(),
                pair: Some((key.into(), value).into()),
            }))
        }
    }

    pub fn new_hmset(table: impl Into<String>, pairs: Vec<KvPair>) -> Self {
        Self {
            request_data: Some(RequestData::Hmset(Hmset {
                table: table.into(),
                pairs,
            }))
        }
    }

    pub fn new_hexists(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hexists(Hexists {
                table: table.into(),
                key: key.into(),
            }))
        }
    }

    pub fn new_hmexists(table: impl Into<String>, keys: Vec<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hmexists(Hmexists {
                table: table.into(),
                keys
            }))
        }
    }

    pub fn new_hdelete(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hdelete(Hdelete {
                table: table.into(),
                key: key.into(),
            }))
        }
    }

    pub fn new_hmdelete(table: impl Into<String>, keys: Vec<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hmdelete(Hmdelete {
                table: table.into(),
                keys
            }))
        }
    }

    pub fn new_hget_all(table: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(Hgetall {
                table: table.into()
            })),
        }
    }

    pub fn subscribe(topic: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Subscribe (Subscribe { 
                topic: topic.into() 
            })),
        }
    }

    pub fn unsubscribe(topic: impl Into<String>, id: u32) -> Self {
        Self {
            request_data: Some(RequestData::Unsubscribe(Unsubscribe {
                topic: topic.into(),
                id,
            })),
        }
    }

    pub fn publish(topic: impl Into<String>, data: Vec<Value>) -> Self {
        Self {
            request_data: Some(RequestData::Publish(Publish {
                topic: topic.into(),
                data,
            }))
        }
    }
}

impl CommandResponse {
    pub fn new(state_code: u32, msg: impl Into<String>, values: Vec<Value>, pairs: Vec<KvPair>, exit: bool) -> Self {
        Self {
            state_code,
            msg: msg.into(),
            values,
            pairs,
            exit
        }
    }

    pub fn ok() -> Self {
        Self {
            state_code: 200,
            msg: "ok".to_string(),
            ..Default::default()
        }
    }

    pub fn exit() -> Self {
        Self {
            state_code: 200,
            msg: "ok".to_string(),
            exit: true,
            ..Default::default()
        }
    }
}

impl TryFrom<&Value> for i64 {
    type Error = KvError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(value::Value::Integer(x)) = value.value {
            return Ok(x);
        }
        Err(KvError::ConvertError)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self {
            value: Some(value::Value::String(value.to_string()))
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            value: Some(value::Value::String(value))
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self {
            value: Some(value::Value::Bool(value))
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self {
            value: Some(value::Value::Integer(value)),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self {
            value: Some(value::Value::Float(value)),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self {
            value: Some(value::Value::Binary(value)),
        }
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = KvError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value = Value::decode(value)?;
        Ok(value)
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = KvError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut buf = Vec::new();
        value.encode(&mut buf)?;
        Ok(buf)
    }
}

impl From<(&str, Value)> for KvPair {
    fn from(value: (&str, Value)) -> Self {
        Self {
            key: value.0.to_string(),
            value: Some(value.1),
        }
    }
}

impl From<(String, Value)> for KvPair {
    fn from(value: (String, Value)) -> Self {
        Self {
            key: value.0,
            value: Some(value.1),
        }
    }
}

impl From<i64> for CommandResponse {
    fn from(value: i64) -> Self {
        Self {
            state_code: 200,
            msg: "ok".to_string(),
            values: vec![value.into()],
            pairs: vec![],
            exit: false,
        }
    }
}

impl From<Value> for CommandResponse {
    fn from(value: Value) -> Self {
        Self {
            state_code: 200,
            msg: "ok".to_string(),
            values: vec![value],
            ..Default::default()
        }
    }
}

impl From<Vec<Value>> for CommandResponse {
    fn from(values: Vec<Value>) -> Self {
        Self {
            state_code: 200,
            msg: "ok".to_string(),
            values,
            ..Default::default()
        }
    }
}

impl From<KvError> for CommandResponse {
    fn from(value: KvError) -> Self {
        let mut res = Self {
            state_code: 400,
            msg: value.to_string(),
            ..Default::default()
        };

        match value {
            KvError::NotFound(_, _) => res.state_code = StatusCode::NOT_FOUND.as_u16() as _,
            KvError::InvalidCommand(_) => res.state_code = StatusCode::BAD_REQUEST.as_u16() as _,
            _ => (),
        }

        res
    }
}

impl From<Vec<KvPair>> for CommandResponse {
    fn from(value: Vec<KvPair>) -> Self {
        Self {
            state_code: 200,
            msg: "ok".to_string(),
            pairs: value,
            ..Default::default()
        }
    }
}