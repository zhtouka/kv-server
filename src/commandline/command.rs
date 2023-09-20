use clap::{Parser, Subcommand};

use crate::CommandRequest;

#[derive(Parser, Debug)]
pub struct Command {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    GET(Get),
    SET(Set),
    EXISTS(Exists),
    DELETE(Delete),
    MGET(MGet),
    MEXISTS(MExists),
    MDELETE(MDelete),
    GETALL(GetAll),
    SUBSCRIBE(Subscribe),
    UNSUBSCRIBE(Unsubscribe),
    PUBLISH(Publish),
}

#[derive(Subcommand, Debug, Clone)]
pub enum Value {
    Text {
        text: String,
    },
    Integer {
        integer: i64,
    },
    Double {
        double: f64
    },
    Boolean {
        #[arg(action = clap::ArgAction::Set)]
        boolean: bool
    },
    Binary {
        binary: Vec<u8>
    }
    // TEXT(String),
    // INTEGER(i64),
    // DOUBLE(f64),
    // BOOLEAN(bool),
    // BINARY(Vec<u8>)
}

#[derive(Parser, Debug)]
pub struct Get {
    pub(crate) table: String,
    pub(crate) key: String,
}

#[derive(Parser, Debug)]
pub struct Set {
    pub(crate) table: String,
    pub(crate) key: String,
    #[command(subcommand)]
    pub(crate) value: Value
}

#[derive(Parser, Debug)]
pub struct Exists {
    pub(crate) table: String,
    pub(crate) key: String,
}

#[derive(Parser, Debug)]
pub struct Delete {
    pub(crate) table: String,
    pub(crate) key: String,
}

#[derive(Parser, Debug)]
pub struct Subscribe {
    pub(crate) topic: String,
}

#[derive(Parser, Debug)]
pub struct Unsubscribe {
    pub(crate) topic: String,
    pub(crate) id: u32,
}

#[derive(Parser, Debug)]
pub struct Publish {
    pub(crate) topic: String,
    #[arg(value_parser = parse_value)]
    pub(crate) data: Vec<Value>
}

#[derive(Parser, Debug)]
pub struct MGet {
    pub(crate) table: String,
    pub(crate) keys: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct MExists {
    pub(crate) table: String,
    pub(crate) keys: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct MDelete {
    pub(crate) table: String,
    pub(crate) keys: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct GetAll {
    pub(crate) table: String,
}

// #[derive(Parser, Debug)]
// pub struct SetAll {
//     pub(crate) table: String,
//     pub(crate) keys: Vec<Pair>,
// }

// #[derive(Parser, Debug, Clone)]
// pub struct Pair {
//     pub(crate) key: String,
//     #[arg(value_parser = parse_value)]
//     pub(crate) value: Value
// }

fn parse_value(s: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let val = s.splitn(2, "@@").collect::<Vec<_>>();
    match val[0] {
        "s" => Ok(Value::Text { text: val[1].to_string() }),
        "i" => Ok(Value::Integer { integer: val[1].parse::<i64>()? }),
        "d" => Ok(Value::Double { double: val[1].parse::<f64>()? }),
        "f" => Ok(Value::Boolean { boolean: val[1].parse::<bool>()? }),
        "b" => Ok(Value::Binary { binary: val[1].as_bytes().to_vec() }),
        _ => Ok(Value::Text { text: s.to_string() }),
    }
} 

pub enum CommandType {
    Unary(CommandRequest),
    Stream(CommandRequest),
}

pub fn get_command() -> CommandType {
    let command = Command::parse();

    match command.subcommand {
        SubCommand::GET(x) => CommandType::Unary(x.into()),
        SubCommand::SET(x) => CommandType::Unary(x.into()),
        SubCommand::EXISTS(x) => CommandType::Unary(x.into()),
        SubCommand::DELETE(x) => CommandType::Unary(x.into()),
        SubCommand::MGET(x) => CommandType::Unary(x.into()),
        SubCommand::MEXISTS(x) => CommandType::Unary(x.into()),
        SubCommand::MDELETE(x) => CommandType::Unary(x.into()),
        SubCommand::GETALL(x) => CommandType::Unary(x.into()),
        SubCommand::SUBSCRIBE(x) => CommandType::Stream(x.into()),
        SubCommand::UNSUBSCRIBE(x) => CommandType::Unary(x.into()),
        SubCommand::PUBLISH(x) => CommandType::Unary(x.into()),        
    }
}