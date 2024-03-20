use std::{collections::BTreeMap, ops::Index};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(Option<String>),
    Array(Option<Vec<Value>>),
    Map(BTreeMap<Value, Value>),
}

impl Value {
    pub fn str(s: &str) -> Self {
        Self::String(Some(s.into()))
    }
}
