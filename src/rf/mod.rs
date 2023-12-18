use std::error;
use std::fmt;
use std::time::Duration;

use crate::config;

pub mod mqtt;
pub mod serial;

pub trait RFClient {
    fn set_network_id(&mut self, network_id: Vec<u8>) -> Result<(), String>;
    fn recv(&mut self) -> Result<Vec<u8>, RecvError>;
    fn recv_timeout(&mut self, timeout: Duration) -> Result<Vec<u8>, RecvTimeoutError>;
    fn send(&mut self, payload: Vec<u8>) -> Result<(), SendError>;
    fn sleep(&mut self) -> Result<(), String>;
}

pub fn new(config: &config::Config) -> Result<Box<dyn RFClient>, String> {
    match &config.mqtt {
        Some(config) => return Ok(Box::new(mqtt::new(&config)?)),
        None => {}
    }
    match &config.serial {
        Some(config) => return Ok(Box::new(serial::new(&config)?)),
        None => {}
    }
    Err("no client configured".to_string())
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RecvError {
    msg: String,
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl error::Error for RecvError {}

impl From<String> for RecvError {
    fn from(err: String) -> RecvError {
        RecvError { msg: err }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RecvTimeoutError {
    Timeout,
    Error { msg: String },
}

impl fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecvTimeoutError::Timeout => "timed out waiting on receive operation".fmt(f),
            RecvTimeoutError::Error { msg } => msg.fmt(f),
        }
    }
}

impl error::Error for RecvTimeoutError {}

impl From<RecvError> for RecvTimeoutError {
    fn from(err: RecvError) -> RecvTimeoutError {
        RecvTimeoutError::Error { msg: err.msg }
    }
}
impl From<String> for RecvTimeoutError {
    fn from(err: String) -> RecvTimeoutError {
        RecvTimeoutError::Error { msg: err }
    }
}

impl RecvTimeoutError {
    /// Returns `true` if the receive operation timed out.
    pub fn is_timeout(&self) -> bool {
        match self {
            RecvTimeoutError::Timeout => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SendError {
    msg: String,
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl error::Error for SendError {}

impl From<String> for SendError {
    fn from(err: String) -> SendError {
        SendError { msg: err }
    }
}
