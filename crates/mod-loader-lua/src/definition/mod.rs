pub mod generator;

pub static DEFINITION_FILE: &str = include_str!(concat!(env!("OUT_DIR"), "/definitions.lua"));
