use crate::config;
use crate::connect::{filter, from_bytes, send_cmd, Assert, Cmd, ConnectError, Metadata};
use crate::rf::RFClient;
use deku::prelude::*;
use std::fmt::Error;
use std::time::Duration;
use std::{fmt, result};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SetExternalTemperatureMsg {
    data: [u8; 9],
    #[deku(endian = "big")]
    temperature: i16,
}

impl fmt::Display for SetExternalTemperatureMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), Error> {
        return Ok(write!(f, "SetExternalTemperatureMsg").expect("TODO: panic data"));
    }
}

impl Assert for SetExternalTemperatureMsg {
    fn assert(&self) -> bool {
        // Doesn't seems to have length.
        true
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct ExternalTemperatureInitMsg {
    #[deku(count = "2")]
    data: Vec<u8>,
}

impl fmt::Display for ExternalTemperatureInitMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), Error> {
        return Ok(write!(f, "ExternalTemperatureInitMsg").expect("TODO: panic data"));
    }
}

impl Assert for ExternalTemperatureInitMsg {
    fn assert(&self) -> bool {
        // self.len as usize == 0x11 // length is expected to represent the msg length(17)
        true
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SetExternalTemperatureReplyMsg {
    len: u8,
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    #[deku(count = "2")]
    data: Vec<u8>,
}

impl fmt::Display for SetExternalTemperatureReplyMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), Error> {
        return Ok(write!(f, "SetExternalTemperatureReplyMsg").expect("TODO: panic data"));
    }
}

impl Assert for SetExternalTemperatureReplyMsg {
    fn assert(&self) -> bool {
        self.len as usize == 0x08 // length is expected to represent the msg length (8) = total - metadata len
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SondeAssociationAnnounceMessage {
    #[deku(count = "0")]
    pub data: Vec<u8>,
}

impl fmt::Display for SondeAssociationAnnounceMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), Error> {
        return Ok(write!(f, "SondeAssociationAnnounceMessage").expect("TODO: panic data"));
    }
}

impl Assert for SondeAssociationAnnounceMessage {
    fn assert(&self) -> bool {
        // self.len as usize == 0x06 // length is expected to represent the msg length(06)
        true
    }
}

pub fn send_temperature(
    rf: &mut Box<dyn RFClient>,
    config: &mut config::Frisquet,
    temperature: i16,
) -> Result<(Metadata, SetExternalTemperatureReplyMsg), ConnectError> {
    rf.set_network_id(Vec::from(config.network_id()?))?;

    let req_id = config.next_req_id()?;

    send_cmd(
        rf,
        0x20, // from
        0x80, // to
        config.association_id()?,
        req_id,
        1,
        03,
        &SetExternalTemperatureMsg {
            data: [156, 84, 0, 4, 160, 41, 0, 1, 2],
            temperature,
        },
    )?;

    loop {
        match filter(
            &rf.recv_timeout(Duration::new(15, 0))?,
            0x80,
            0x20,
            config.association_id()?,
            req_id,
        )? {
            Some(payload) => {
                let (meta, data) = from_bytes(&payload)?;
                println!("RECV {} {}", meta, data);
                return Ok((meta, data));
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_message() {
        let payload = hex::decode("118020ba4001179c540004a029000102005c").unwrap();
        let (meta, data) = from_bytes(&payload).unwrap();
        let x: SetExternalTemperatureMsg = data;

        assert_eq!(
            meta,
            Metadata {
                length: 17,
                to_addr: 128,
                from_addr: 32,
                association_id: 186,
                request_id: 64,
                control: 1,
                msg_type: 23,
            }
        );
        assert_eq!(
            x,
            SetExternalTemperatureMsg {
                data: [156, 84, 0, 4, 160, 41, 0, 1, 2],
                temperature: 92,
            }
        );
    }

    #[test]
    fn test_announce_response() {
        let payload = hex::decode("06802020948241").unwrap();

        let (meta, data) = from_bytes(&payload).unwrap();
        let x: SondeAssociationAnnounceMessage = data;
        assert_eq!(
            meta,
            Metadata {
                length: 6,
                to_addr: 128,
                from_addr: 32,
                association_id: 32,
                request_id: 148,
                control: 130,
                msg_type: 65,
            }
        );
        assert_eq!(x, SondeAssociationAnnounceMessage { data: vec![] });
    }

    #[test]
    fn test_set_temperature_response() {
        let payload = hex::decode("0f2080ba408117082304051131172803").unwrap();

        let (meta, data) = from_bytes(&payload).unwrap();
        let x: SetExternalTemperatureReplyMsg = data;
        assert_eq!(
            meta,
            Metadata {
                length: 15,
                to_addr: 32,
                from_addr: 128,
                association_id: 186,
                request_id: 64,
                control: 129,
                msg_type: 23,
            }
        );
        assert_eq!(
            x,
            SetExternalTemperatureReplyMsg {
                len: 8,
                year: 35,
                month: 4,
                day: 5,
                hour: 17,
                minute: 49,
                second: 23,
                data: [40, 3].to_vec(),
            }
        );
    }

    #[test]
    fn test_init() {
        let payload = hex::decode("088020830001430000").unwrap();

        let (meta, data) = from_bytes(&payload).unwrap();

        let x: ExternalTemperatureInitMsg = data;
        assert_eq!(
            meta,
            Metadata {
                length: 8,
                to_addr: 128,
                from_addr: 32,
                association_id: 131,
                request_id: 0,
                control: 1,
                msg_type: 67,
            }
        );
        assert_eq!(x, ExternalTemperatureInitMsg { data: vec![0, 0] });
    }
}
