use std::{error, fmt};

pub mod homeassistant;

pub trait ExternalTemperatureSource {
    fn get(&mut self) -> Result<f32, ExternalTemperatureErr>;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ExternalTemperatureErr {
    msg: String,
}

impl fmt::Display for ExternalTemperatureErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl error::Error for ExternalTemperatureErr {}

impl From<String> for ExternalTemperatureErr {
    fn from(err: String) -> ExternalTemperatureErr {
        ExternalTemperatureErr { msg: err }
    }
}
