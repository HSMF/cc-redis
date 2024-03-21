use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::Mutex;
use serde::Serialize;

use crate::{case_insensitive::CaseInsensitive, serializer::to_bytes, value::Value};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone)]
struct Entry {
    value: Value,
    expiry: Option<u128>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        let Some(expiry) = self.expiry else {
            return false;
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_millis();
        now > expiry
    }

    fn new(value: Value) -> Self {
        Self {
            value,
            expiry: None,
        }
    }

    fn expires_in(&mut self, ms: u128) -> &mut Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_millis();
        self.expiry = Some(now + ms);
        self
    }
}

#[derive(Debug)]
pub struct App {
    store: Arc<Mutex<BTreeMap<Value, Entry>>>,
    config: Mutex<BTreeMap<String, String>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(BTreeMap::new())),
            config: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn prune_expired(&self) {
        let mut store = self.store.lock();
        let expired: Vec<_> = store
            .iter()
            .filter_map(|(k, v)| v.is_expired().then_some(k).cloned())
            .collect();

        for e in expired {
            store.remove(&e);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("-FAILURE")]
    Failure,
    #[error("-ERR {0}")]
    Generic(String),
    #[error("-ERR {0}")]
    GenericStatic(&'static str),
    #[error("-INVALIDREQ {0}")]
    InvalidReq(&'static str),
    #[error("-UNKNOWN {0}")]
    UnknownCommand(String),
    #[error("-WRONGTYPE {0}")]
    TypeError(String),
}

type Resp<T> = Result<T, Error>;

trait ArgParse: Sized {
    fn from_args(args: &[Value]) -> Result<Self, Error>;
}

struct SetArgs {
    key: Value,
    val: Value,
    expiry: Option<i64>,
}

impl ArgParse for SetArgs {
    fn from_args(args: &[Value]) -> Result<Self, Error> {
        let (key, args) = args
            .split_first()
            .ok_or(Error::GenericStatic("set is missing key"))?;
        let key = key.to_owned();
        let (val, args) = args
            .split_first()
            .ok_or(Error::GenericStatic("set is missing value argument"))?;
        let val = val.to_owned();

        let mut out = SetArgs {
            key,
            val,
            expiry: None,
        };

        let mut args = args.iter();

        while let Some(arg) = args.next() {
            if arg.get_str().is_some_and(|x| CaseInsensitive(x) == "px") {
                let expiry = args
                    .next()
                    .ok_or(Error::GenericStatic("PX expects expiry."))?;

                let expiry = expiry
                    .get_str()
                    .and_then(|x| x.parse::<i64>().ok())
                    .ok_or(Error::TypeError("expiry must be an int".into()))?;

                out.expiry = Some(expiry)
            }
        }

        Ok(out)
    }
}

enum ConfigArgs {
    Get(String),
    Set(String, String),
}

impl ArgParse for ConfigArgs {
    fn from_args(args: &[Value]) -> Result<Self, Error> {
        let (verb, args) = args
            .split_first()
            .ok_or(Error::GenericStatic("config requires verb (get/set)"))?;

        let verb = verb
            .get_str()
            .ok_or(Error::GenericStatic("verb must be string"))?
            .to_ascii_lowercase();

        match verb.as_str() {
            "get" => {
                if args.len() != 1 {
                    return Err(Error::GenericStatic(
                        "config get requires exactly one parameter",
                    ));
                }
                let key = args[0]
                    .get_str()
                    .ok_or(Error::GenericStatic("config get key must be string"))?;
                Ok(Self::Get(key.clone()))
            }
            "set" => {
                if args.len() != 2 {
                    return Err(Error::GenericStatic(
                        "config set requires exactly two parameters",
                    ));
                }

                let key = args[0]
                    .get_str()
                    .ok_or(Error::GenericStatic("config set key must be string"))?
                    .clone();
                let value = args[1]
                    .get_str()
                    .ok_or(Error::GenericStatic("config set value must be string"))?
                    .clone();

                Ok(Self::Set(key, value))
            }
            _ => Err(Error::GenericStatic("config requires verb (get/set)")),
        }
    }
}

impl App {
    pub fn set_config(&self, key: String, value: String) {
        self.config.lock().insert(key, value);
    }

    pub async fn ping(&self) -> Resp<impl Serialize> {
        Ok("PONG")
    }

    pub async fn echo(&self, argv: &[Value]) -> Resp<impl Serialize> {
        let [v] = argv else {
            return Err(Error::InvalidReq("echo expects exactly one argument"));
        };
        Ok(v.clone())
    }

    pub async fn set(&self, argv: &[Value]) -> Resp<impl Serialize> {
        let args = SetArgs::from_args(argv)?;

        let mut map = self.store.lock();
        let mut entry = Entry::new(args.val);

        if let Some(expiry) = args.expiry.and_then(|x| x.try_into().ok()) {
            entry.expires_in(expiry);
        }

        map.insert(args.key, entry);

        Ok("OK")
    }

    pub async fn get(&self, argv: &[Value]) -> Resp<impl Serialize> {
        let [k] = argv else {
            return Err(Error::InvalidReq("get expects exactly one argument"));
        };
        let map = self.store.lock();
        let v = map.get(k).cloned().unwrap_or_default();

        if v.is_expired() {
            return Ok(Value::Null);
        }

        Ok(v.value)
    }

    pub async fn config(&self, argv: &[Value]) -> Resp<impl Serialize> {
        let args = ConfigArgs::from_args(argv)?;

        match args {
            ConfigArgs::Get(k) => {
                let config_value = Value::String(self.config.lock().get(&k).cloned());
                Ok(Value::Array(Some(vec![
                    Value::String(Some(k)),
                    config_value,
                ])))
            }
            ConfigArgs::Set(key, value) => {
                self.set_config(key, value);
                Ok(Value::str("OK"))
            }
        }
    }

    async fn dispatch_inner(&self, arg: Value) -> Resp<Vec<u8>> {
        let Value::Array(Some(argv)) = arg else {
            return Err(Error::InvalidReq("command must be an array"));
        };

        let Some((cmd, args)) = argv.split_first() else {
            return Err(Error::InvalidReq("argv must not be empty"));
        };
        let Value::String(Some(command)) = cmd else {
            return Err(Error::TypeError("command must be a string".into()));
        };

        match command.to_lowercase().as_str() {
            "ping" => self.ping().await.to_bytes(),
            "echo" => self.echo(args).await.to_bytes(),
            "set" => self.set(args).await.to_bytes(),
            "get" => self.get(args).await.to_bytes(),
            "config" => self.config(args).await.to_bytes(),
            _ => Err(Error::UnknownCommand(command.to_owned())),
        }
    }

    pub async fn dispatch_command(&self, arg: Value) -> Vec<u8> {
        match self.dispatch_inner(arg).await {
            Ok(i) => i,
            Err(e) => (e.to_string() + "\r\n").into_bytes(),
        }
    }
}

trait ToBytes {
    fn to_bytes(self) -> Result<Vec<u8>, Error>;
}

impl<T> ToBytes for Result<T, Error>
where
    T: Serialize,
{
    fn to_bytes(self) -> Result<Vec<u8>, Error> {
        match self {
            Ok(ok) => to_bytes(&ok).map_err(|_| Error::GenericStatic("failed to serialize")),
            Err(i) => Err(i),
        }
    }
}
