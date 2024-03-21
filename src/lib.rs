pub mod serializer;
pub mod deserializer;
pub mod value;
pub mod commands;
mod case_insensitive;
mod rdb;

pub fn add(x: i32, y: i32) -> i32 {
    x + y
}
