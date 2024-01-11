use deku::prelude::*;
use std::error;
use std::fmt;
use std::fmt::{Debug, Display};

use crate::config::ConfigError;
use crate::rf::{RFClient, RecvError, RecvTimeoutError, SendError};

pub mod area;
pub mod boiler;
pub mod data1;
pub mod data2;
pub mod data3;
pub mod data4;
pub mod date;
pub mod pair;
pub mod sensors;
pub mod promiscuous;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(endian = "big")]
pub struct Metadata {
    pub length: u8,
    pub to_addr: u8,
    pub from_addr: u8,
    pub association_id: u8,
    pub request_id: u8,
    pub control: u8,
    pub msg_type: u8,
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:02x}) 0x{:02x} > 0x{:02x} [{:02x}|{:02x}] {:02x} {:02x}, ",
            self.length,
            self.from_addr,
            self.to_addr,
            self.association_id,
            self.request_id,
            self.control,
            self.msg_type,
        )
    }
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(endian = "big")]
struct DropMsg {}

impl fmt::Display for DropMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DROPPED")
    }
}

impl Assert for DropMsg {
    fn assert(&self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, DekuWrite)]
#[deku(endian = "big")]
struct Cmd {
    pub data: Vec<u8>,
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.data.clone()))
    }
}

pub fn send_cmd<T>(
    rf: &mut Box<dyn RFClient>,
    from: u8, // TODO enum
    to: u8,
    association_id: u8,
    request_id: u8,
    control: u8,
    msg_type: u8,
    cmd: &T,
) -> Result<(), ConnectError>
where
    T: DekuContainerWrite + Display,
{
    let mut data = cmd.to_bytes()?;

    let metadata = Metadata {
        length: u8::try_from(data.len() + 6).map_err(|e| e.to_string())?,
        to_addr: to,
        from_addr: from,
        association_id: association_id,
        request_id: request_id,
        control: control,
        msg_type: msg_type,
    };
    let mut payload = metadata.to_bytes()?;
    payload.append(&mut data);
    let data = hex::encode(payload.clone());

    println!("SEND {}{}", metadata, data);

    Ok(rf.send(payload)?)
}

pub fn filter(
    payload: &Vec<u8>,
    from: u8,
    to: u8,
    association_id: u8,
    request_id: u8,
) -> Result<Option<&Vec<u8>>, DekuError> {
    let (_, meta) = Metadata::from_bytes((payload, 0))?;

    if meta.from_addr != from {
        return Ok(None);
    }
    if meta.to_addr != to {
        return Ok(None);
    }
    if meta.association_id != association_id {
        return Ok(None);
    }
    if meta.request_id != request_id {
        return Ok(None);
    }

    Ok(Some(payload))
}

pub fn from_bytes<'a, T>(payload: &'a Vec<u8>) -> Result<(Metadata, T), DekuError>
where
    T: DekuContainerRead<'a> + Assert,
{
    let (_, meta) = Metadata::from_bytes((payload, 0))?;
    let (_, data) = T::from_bytes((&payload[7..], 0))?;
    assert!(data.assert());
    Ok((meta, data))
}

fn format_day(data: [u8; 6]) -> String {
    let mut out = String::new();
    for d in data {
        out.push_str(
            format!("{:08b}", d)
                .chars()
                .rev()
                .collect::<String>()
                .as_str(),
        )
    }

    out
}

pub trait Assert {
    fn assert(&self) -> bool;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConnectError {
    msg: String,
}

impl ConnectError {
    pub fn new(msg: &str) -> ConnectError {
        ConnectError { msg: msg.into() }
    }
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.msg, f)
    }
}

impl error::Error for ConnectError {}

impl From<String> for ConnectError {
    fn from(err: String) -> ConnectError {
        ConnectError { msg: err }
    }
}
impl From<SendError> for ConnectError {
    fn from(err: SendError) -> ConnectError {
        ConnectError {
            msg: err.to_string(),
        }
    }
}
impl From<RecvError> for ConnectError {
    fn from(err: RecvError) -> ConnectError {
        ConnectError {
            msg: err.to_string(),
        }
    }
}
impl From<RecvTimeoutError> for ConnectError {
    fn from(err: RecvTimeoutError) -> ConnectError {
        ConnectError {
            msg: err.to_string(),
        }
    }
}
impl From<DekuError> for ConnectError {
    fn from(err: DekuError) -> ConnectError {
        ConnectError {
            msg: err.to_string(),
        }
    }
}
impl From<ConfigError> for ConnectError {
    fn from(err: ConfigError) -> ConnectError {
        ConnectError {
            msg: err.to_string(),
        }
    }
}
