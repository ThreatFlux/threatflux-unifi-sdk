//! Declarative configuration support.

pub mod parser;
pub mod schema;

pub use parser::{load_config, parse_json, parse_yaml};
pub use schema::*;
