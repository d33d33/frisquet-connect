use deku::prelude::*;
use hex;
use std::fmt;

use crate::config;
use crate::connect::{filter, from_bytes, send_cmd, Cmd, ConnectError, Metadata};
use crate::rf::RFClient;

use super::Assert;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct Data1Msg {
    len: u8,
    #[deku(count = "len")]
    items: Vec<u8>,
}

impl fmt::Display for Data1Msg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes().map(hex::encode) {
            Ok(data) => {
                write!(f, "{}", data)
            }
            Err(_) => write!(f, "ERROR"),
        }
    }
}

impl Assert for Data1Msg {
    fn assert(&self) -> bool {
        true
    }
}

pub fn connect_data1(
    rf: &mut Box<dyn RFClient>,
    config: &mut config::Frisquet,
) -> Result<(Metadata, Data1Msg), ConnectError> {
    rf.set_network_id(Vec::from(config.network_id()?))?;

    let req_id = config.next_req_id()?;
    // 79fc001c
    send_cmd(
        rf,
        0x7e, // from
        0x80, // to
        config.association_id()?,
        req_id,
        01,
        03,
        &Cmd {
            data: vec![0x79, 0xfc, 0x00, 0x1c],
        },
    )?;

    loop {
        match filter(&rf.recv()?, 0x80, 0x7e, config.association_id()?, req_id)? {
            Some(payload) => {
                let (meta, data) = from_bytes(&payload)?;
                println!("RECV {}{}", meta, data);
                return Ok((meta, data));
            }
            None => {}
        }
    }
}
