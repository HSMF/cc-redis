use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(Option<String>),
    Array(Option<Vec<Value>>),
    Map(BTreeMap<Value, Value>),
    #[default]
    Null,
}

impl Value {
    pub fn str(s: &str) -> Self {
        Self::String(Some(s.into()))
    }

    pub fn to_int(self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn to_bool(self) -> Option<bool> {
        match self {
            Self::Bool(i) => Some(i),
            _ => None,
        }
    }

    pub fn to_str(self) -> Option<String> {
        match self {
            Self::String(i) => i,
            _ => None,
        }
    }

    pub fn to_arr(self) -> Option<Vec<Value>> {
        match self {
            Self::Array(i) => i,
            _ => None,
        }
    }

    pub fn to_map(self) -> Option<BTreeMap<Value, Value>> {
        match self {
            Self::Map(i) => Some(i),
            _ => None,
        }
    }

    pub fn get_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_str(&self) -> Option<&String> {
        match self {
            Self::String(i) => i.as_ref(),
            _ => None,
        }
    }

    pub fn get_arr(&self) -> Option<&Vec<Value>> {
        match self {
            Self::Array(i) => i.as_ref(),
            _ => None,
        }
    }

    pub fn get_map(&self) -> Option<&BTreeMap<Value, Value>> {
        match self {
            Self::Map(i) => Some(i),
            _ => None,
        }
    }
}
