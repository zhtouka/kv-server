#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandRequest {
    #[prost(
        oneof = "command_request::RequestData",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12"
    )]
    pub request_data: ::core::option::Option<command_request::RequestData>,
}
/// Nested message and enum types in `CommandRequest`.
pub mod command_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum RequestData {
        #[prost(message, tag = "1")]
        Hget(super::Hget),
        #[prost(message, tag = "2")]
        Hmget(super::Hmget),
        #[prost(message, tag = "3")]
        Hset(super::Hset),
        #[prost(message, tag = "4")]
        Hmset(super::Hmset),
        #[prost(message, tag = "5")]
        Hexists(super::Hexists),
        #[prost(message, tag = "6")]
        Hmexists(super::Hmexists),
        #[prost(message, tag = "7")]
        Hdelete(super::Hdelete),
        #[prost(message, tag = "8")]
        Hmdelete(super::Hmdelete),
        #[prost(message, tag = "9")]
        Hgetall(super::Hgetall),
        #[prost(message, tag = "10")]
        Subscribe(super::Subscribe),
        #[prost(message, tag = "11")]
        Unsubscribe(super::Unsubscribe),
        #[prost(message, tag = "12")]
        Publish(super::Publish),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandResponse {
    #[prost(uint32, tag = "1")]
    pub state_code: u32,
    #[prost(string, tag = "2")]
    pub msg: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub values: ::prost::alloc::vec::Vec<Value>,
    #[prost(message, repeated, tag = "4")]
    pub pairs: ::prost::alloc::vec::Vec<KvPair>,
    #[prost(bool, tag = "5")]
    pub exit: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hget {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub key: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hmget {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hset {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub pair: ::core::option::Option<KvPair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hmset {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub pairs: ::prost::alloc::vec::Vec<KvPair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hexists {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub key: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hmexists {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hdelete {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub key: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hmdelete {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hgetall {
    #[prost(string, tag = "1")]
    pub table: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Subscribe {
    #[prost(string, tag = "1")]
    pub topic: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Unsubscribe {
    #[prost(string, tag = "1")]
    pub topic: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub id: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Publish {
    #[prost(string, tag = "1")]
    pub topic: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub data: ::prost::alloc::vec::Vec<Value>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KvPair {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<Value>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(oneof = "value::Value", tags = "1, 2, 3, 4, 5")]
    pub value: ::core::option::Option<value::Value>,
}
/// Nested message and enum types in `Value`.
pub mod value {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        String(::prost::alloc::string::String),
        #[prost(bytes, tag = "2")]
        Binary(::prost::alloc::vec::Vec<u8>),
        #[prost(int64, tag = "3")]
        Integer(i64),
        #[prost(double, tag = "4")]
        Float(f64),
        #[prost(bool, tag = "5")]
        Bool(bool),
    }
}
