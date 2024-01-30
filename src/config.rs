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
    pub sonde: Option<Frisquet>,
    pub home_assistant: Option<HAConfig>,

    pub serial: Option<Serial>,
    pub mqtt: Option<MQTT>,
    pub area1: Option<Area>,

    #[serde(skip)]
    path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frisquet {
    pub send_init: Option<bool>,
    #[serde(serialize_with = "slice_as_hex", deserialize_with = "slice_from_hex")]
    pub network_id: Option<[u8; 4]>,
    #[serde(serialize_with = "u8_as_hex", deserialize_with = "u8_from_hex")]
    pub association_id: Option<u8>,
    #[serde(serialize_with = "u8_as_hex", deserialize_with = "u8_from_hex")]
    pub request_id: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HAConfig {
    pub host: String,
    pub port: u16,
    pub token: String,
    pub entity_id: String,
    pub temperature_field: String,
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Area {
    pub comfort: f32,
    pub reduced: f32,
    pub frost: f32,
    pub mode: String,
    pub boost: bool,
    pub r#override: String,
    pub monday: Vec<Prog>,
    pub tuesday: Vec<Prog>,
    pub wednesday: Vec<Prog>,
    pub thursday: Vec<Prog>,
    pub friday: Vec<Prog>,
    pub saturday: Vec<Prog>,
    pub sunday: Vec<Prog>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prog {
    pub timeframe: String,
    pub mode: String,
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
    pub fn sonde(&mut self) -> Result<&mut Frisquet, ConfigError> {
        match &mut self.sonde {
            Some(frisquet) => Ok(frisquet),
            None => Err(ConfigError::new("missing required config: sonde")),
        }
    }

    pub fn home_assistant(&mut self) -> Result<&mut HAConfig, ConfigError> {
        match &mut self.home_assistant {
            Some(home_assistant) => Ok(home_assistant),
            None => Err(ConfigError::new("missing required config: sonde")),
        }
    }

    pub fn area1(&mut self) -> Result<&mut Area, ConfigError> {
        match &mut self.area1 {
            Some(area) => Ok(area),
            None => Err(ConfigError::new("missing required config: area1")),
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

impl Area {
    pub fn comfort(&self) -> Result<u8, ConfigError> {
        let comfort = (self.comfort * 10.0).round() as i32 - 50;
        if comfort < 0 {
            Err(ConfigError::new("comfort temp minimum is 5.0"))
        } else if comfort > 255 {
            Err(ConfigError::new("comfort temp maximum is 30.5"))
        } else if comfort % 5 != 0 {
            Err(ConfigError::new("comfort temp should be 0.5 inc"))
        } else {
            Ok(comfort as u8)
        }
    }
    pub fn reduced(&self) -> Result<u8, ConfigError> {
        let reduced = (self.reduced * 10.0).round() as i32 - 50;
        if reduced < 0 {
            Err(ConfigError::new("reduced temp minimum is 5.0"))
        } else if reduced > 255 {
            Err(ConfigError::new("reduced temp maximum is 30.5"))
        } else if reduced % 5 != 0 {
            Err(ConfigError::new("reduced temp should be 0.5 inc"))
        } else {
            Ok(reduced as u8)
        }
    }
    pub fn frost(&self) -> Result<u8, ConfigError> {
        let frost = (self.frost * 10.0).round() as i32 - 50;
        if frost < 0 {
            Err(ConfigError::new("frost temp minimum is 5.0"))
        } else if frost > 255 {
            Err(ConfigError::new("frost temp maximum is 30.5"))
        } else if frost % 5 != 0 {
            Err(ConfigError::new("frost temp should be 0.5 inc"))
        } else {
            Ok(frost as u8)
        }
    }
    pub fn mode(&self) -> Result<u8, ConfigError> {
        match self.mode.as_str() {
            "auto" => Ok(0x05),
            "comfort" => Ok(0x06),
            "reduced" => Ok(0x07),
            "frost" => Ok(0x08),
            x => Err(ConfigError::new(format!("invalid mode: {}", x).as_str())),
        }
    }
    pub fn mode_is_auto(&self) -> Result<bool, ConfigError> {
        Ok(self.mode()? == 0x05)
    }
    pub fn mode_is_comfort(&self) -> Result<bool, ConfigError> {
        Ok(self.mode()? == 0x06)
    }
    pub fn comfort_override(&self) -> Result<Option<bool>, ConfigError> {
        match self.r#override.as_str() {
            "comfort" => Ok(Some(true)),
            "reduced" => Ok(Some(false)),
            "none" => Ok(None),
            x => Err(ConfigError::new(format!("invalid mode: {}", x).as_str())),
        }
    }
    pub fn monday(&self) -> Result<[u8; 6], ConfigError> {
        to_mode_prog(&self.monday)
    }
    pub fn tuesday(&self) -> Result<[u8; 6], ConfigError> {
        to_mode_prog(&self.tuesday)
    }
    pub fn wednesday(&self) -> Result<[u8; 6], ConfigError> {
        to_mode_prog(&self.wednesday)
    }
    pub fn thursday(&self) -> Result<[u8; 6], ConfigError> {
        to_mode_prog(&self.thursday)
    }
    pub fn friday(&self) -> Result<[u8; 6], ConfigError> {
        to_mode_prog(&self.friday)
    }
    pub fn saturday(&self) -> Result<[u8; 6], ConfigError> {
        to_mode_prog(&self.saturday)
    }
    pub fn sunday(&self) -> Result<[u8; 6], ConfigError> {
        to_mode_prog(&self.sunday)
    }
}

pub fn to_mode_prog(prog: &Vec<Prog>) -> Result<[u8; 6], ConfigError> {
    let mut res: [u8; 6] = [0; 6];
    for s in prog {
        match s.mode.as_str() {
            "comfort" => {
                let idx = time_indexes(s.timeframe.as_str()).map_err(|_| {
                    ConfigError::new("timeframe should be 12h00-14h30 format, 30 min increment")
                })?;
                for n in idx.0..idx.1 {
                    res[(n / 8) as usize] = res[(n / 8) as usize] | 1 << (n % 8);
                }
            }
            "reduced" => {
                continue; // reduced is default
            }
            x => {
                return Err(ConfigError::new(format!("invalid mode: {}", x).as_str()));
            }
        }
    }
    Ok(res)
}

fn time_indexes(timeframe: &str) -> Result<(u8, u8), ()> {
    let mut segments = timeframe.split("-");
    let start = segments.next().ok_or(())?;
    let end = segments.next().ok_or(())?;
    if segments.next().is_some() {
        return Err(());
    }

    let start = time_index(start)?;
    let mut end = time_index(end)?;

    if start > 0 && end == 0 {
        // end is 00h00
        end = time_index("23h30")? + 1;
    }

    if end < start {
        return Err(());
    }

    Ok((start, end))
}

fn time_index(time: &str) -> Result<u8, ()> {
    let mut segments = time.split("h");
    let hours = segments.next().ok_or(())?;
    let minutes = segments.next().ok_or(())?;
    if segments.next().is_some() {
        return Err(());
    }

    let hours: u8 = hours.parse().map_err(|_| ())?;
    if hours >= 24 {
        return Err(());
    }
    let minutes: u8 = minutes.parse().map_err(|_| ())?;
    if minutes != 0 && minutes != 30 {
        return Err(());
    }

    Ok(hours * 2 + minutes / 30)
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
