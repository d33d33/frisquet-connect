use deku::prelude::*;
use hex;
use std::fmt;

use crate::config;
use crate::connect::{filter, from_bytes, send_cmd, Cmd, ConnectError, Metadata};
use crate::rf::RFClient;

use super::Assert;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct Data4Msg {
    len: u8,
    #[deku(count = "len")]
    items: Vec<u8>,
}

impl fmt::Display for Data4Msg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes().map(hex::encode) {
            Ok(data) => {
                write!(f, "{}", data)
            }
            Err(_) => write!(f, "ERROR"),
        }
    }
}

impl Assert for Data4Msg {
    fn assert(&self) -> bool {
        true
    }
}

pub fn connect_data4(
    rf: &mut Box<dyn RFClient>,
    config: &mut config::Frisquet,
) -> Result<(Metadata, Data4Msg), ConnectError> {
    rf.set_network_id(Vec::from(config.network_id()?))?;

    let req_id = config.next_req_id()?;
    // a0f000159c400001020000
    send_cmd(
        rf,
        0x7e, // from
        0x80, // to
        config.association_id()?,
        req_id,
        01,
        0x17,
        &Cmd {
            data: vec![
                0xa0, 0xf0, 0x00, 0x15, 0x9c, 0x40, 0x00, 0x01, 0x02, 0x00, 0x00,
            ],
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
