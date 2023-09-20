use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Command {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    GET(Get),
    SET(Set),
    PUBLISH(Publish),
    MSET(MSet),
}

#[derive(Parser, Debug)]
pub struct Get {
    table: String,
    key: String,
}

#[derive(Parser, Debug)]
pub struct Set {
    table: String,
    key: String,
    #[command(subcommand)]
    value: Value,
}

#[derive(Parser, Debug)]
pub struct Publish {
    topic: String,
    #[arg(value_parser = parse_value)]
    data: Vec<Value>,
}

#[derive(Parser, Debug)]
pub struct MSet {
    table: String,
    #[arg(value_parser = parse_pair)]
    pairs: Vec<Pair>
}

#[derive(Debug, Parser, Clone)]
pub struct  Pair {
    key: String,
    #[command(subcommand)]
    value: Value
}

// k1::i@@123, k1::s@@123, k1::d@@123, k1::f@@false, k1::b@@[1, 2, 3]
fn parse_pair(s: &str) -> Result<Pair, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let key = s.split("::").next().expect("missing key value delimiter");
    let val = s.splitn(2, "@@").collect::<Vec<_>>();
    match val[0] {
        "s" => Ok(Pair { key: key.to_string(), value: Value::Text { text: val[1].to_string() } }),
        "i" => Ok(Pair { key: key.to_string(), value: Value::Integer { integer: val[1].parse::<i64>()? }}),
        "d" => Ok(Pair { key: key.to_string(), value: Value::Double { double: val[1].parse::<f64>()? } }),
        "f" => Ok(Pair { key: key.to_string(), value: Value::Boolean { boolean: val[1].parse::<bool>()? } }),
        "b" => Ok(Pair { key: key.to_string(), value: Value::Binary { binary: val[1].as_bytes().to_vec() } }),
        _ => Ok(Pair { key: key.to_string(), value: Value::Text { text: val[1].to_string() } }),
    }
}

// i@@123, s@@123, d@@123, f@@false, b@@123
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

#[derive(Parser, Debug, Clone)]
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
}

fn main() {
    let value = Command::parse();
    println!("{:?}", value);
}