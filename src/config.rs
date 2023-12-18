use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error;
use std::fmt;
use std::fmt::Debug;
use std::fs::{read_to_string, write};
use std::usize;
use toml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub frisquet: Option<Frisquet>,
    pub serial: Option<Serial>,
    pub mqtt: Option<MQTT>,

    #[serde(skip)]
    path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frisquet {
    #[serde(serialize_with = "slice_as_hex", deserialize_with = "slice_from_hex")]
    pub network_id: Option<[u8; 4]>,
    #[serde(serialize_with = "u8_as_hex", deserialize_with = "u8_from_hex")]
    pub association_id: Option<u8>,
    #[serde(serialize_with = "u8_as_hex", deserialize_with = "u8_from_hex")]
    pub request_id: Option<u8>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Serial {
    pub port: String,
    pub speed: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MQTT {
    pub broker: String,
    pub client_id: String,
    pub cmd_topic: String,
    pub lst_topic: String,
}

pub fn read(path: &str) -> Result<Config, ConfigError> {
    let content = read_to_string(path)?;
    let mut config: Config = toml::from_str(&content)?;
    config.path = path.into();
    Ok(config)
}

impl Config {
    pub fn write(&self) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)?;
        write(&self.path, content)?;
        Ok(())
    }

    pub fn frisquet(&mut self) -> Result<&mut Frisquet, ConfigError> {
        match &mut self.frisquet {
            Some(frisquet) => Ok(frisquet),
            None => Err(ConfigError::new("missing required config: frisquet")),
        }
    }
}

impl Frisquet {
    pub fn network_id(&self) -> Result<Vec<u8>, ConfigError> {
        match self.network_id {
            Some(id) => Ok(Vec::from(id)),
            None => Err(ConfigError::new(
                "missing required config: frisquet.network_id",
            )),
        }
    }
    pub fn association_id(&self) -> Result<u8, ConfigError> {
        match self.association_id {
            Some(id) => Ok(id),
            None => Err(ConfigError::new(
                "missing required config: frisquet.association_id",
            )),
        }
    }
    pub fn next_req_id(&mut self) -> Result<u8, ConfigError> {
        match self.request_id {
            Some(id) => {
                let (req_id, _) = id.overflowing_add(4);
                self.request_id = Some(req_id);
                Ok(req_id)
            }
            None => Err(ConfigError::new(
                "missing required config: frisquet.request_id",
            )),
        }
    }
}

fn slice_as_hex<T, S>(option: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    match option {
        Some(src) => {
            let data = hex::encode(src.as_ref());
            serializer.serialize_str(&data)
        }
        None => serializer.serialize_none(),
    }
}
fn slice_from_hex<'de, const N: usize, D>(deserializer: D) -> Result<Option<[u8; N]>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let option: Option<String> = Deserialize::deserialize(deserializer)?;
    match option {
        Some(src) => {
            println!("{}", src);
            let data = hex::decode(src).map_err(|e| Error::custom(e.to_string()))?;

            let len = data.len();
            let res: [u8; N] = data.try_into().map_err(|_| {
                let expected = format!("[u8; {}]", N);
                Error::invalid_length(len, &expected.as_str())
            })?;

            Ok(Some(res))
        }
        None => Ok(None),
    }
}
fn u8_as_hex<S>(option: &Option<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match option {
        Some(src) => {
            let data = hex::encode(vec![*src]);
            serializer.serialize_str(&data)
        }
        None => serializer.serialize_none(),
    }
}
fn u8_from_hex<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let option: Option<String> = Deserialize::deserialize(deserializer)?;
    match option {
        Some(src) => {
            println!("{}", src);
            let data = hex::decode(src).map_err(|e| Error::custom(e.to_string()))?;

            if data.len() != 1 {
                let expected = format!("u8");
                return Err(Error::invalid_length(data.len(), &expected.as_str()));
            }

            Ok(Some(data[0]))
        }
        None => Ok(None),
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConfigError {
    msg: String,
}

impl ConfigError {
    pub fn new(msg: &str) -> ConfigError {
        ConfigError { msg: msg.into() }
    }
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.msg, f)
    }
}

impl error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> ConfigError {
        ConfigError {
            msg: err.to_string(),
        }
    }
}
impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> ConfigError {
        ConfigError {
            msg: format!("invalid toml: {}", err.to_string()),
        }
    }
}
impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> ConfigError {
        ConfigError {
            msg: format!("fail to serialize toml: {}", err.to_string()),
        }
    }
}
