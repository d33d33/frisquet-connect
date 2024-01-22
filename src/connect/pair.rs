use std::fmt;
use std::time::Duration;

use deku::prelude::*;
use hex;

use crate::connect::{from_bytes, send_cmd, ConnectError, Metadata};

use crate::rf::RFClient;

use super::Assert;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
struct AssociationCmd {
    version: [u8; 4],
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
struct AssociationMsg {
    len: u8,
    network_id: [u8; 4],
}

pub struct Association {
    pub network_id: [u8; 4],
    pub association_id: u8,
    pub request_id: u8,
}

impl fmt::Display for AssociationCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.to_bytes().map(hex::encode).unwrap_or("ERROR".into());
        write!(f, "{}\n    AssociationCmd", data)
    }
}

impl fmt::Display for AssociationMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.to_bytes().map(hex::encode).unwrap_or("ERROR".into());
        write!(f, "{}\n    AssociationMsg", data)?;
        write!(f, "\n\tNetwork ID: {}", hex::encode(self.network_id))
    }
}

impl Assert for AssociationMsg {
    fn assert(&self) -> bool {
        self.len as usize == self.network_id.len() // length is expected to represent the network_id length
    }
}

impl Assert for AssociationCmd {
    fn assert(&self) -> bool {
        // Doesn't seem to have length
        true
    }
}

pub fn connect_association(
    rf: &mut Box<dyn RFClient>,
    from: u8,
) -> Result<Association, ConnectError> {
    let network_id: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff];
    rf.set_network_id(network_id)?;
    println!("Setting network_id to ffffffff"); // TODO log in set_network_id

    println!("Waiting broadcast message");
    let mut res: Option<Association> = None;
    loop {
        match wait_association_msg(rf, Duration::new(5, 0))? {
            Some((meta, data)) => {
                if meta.control != 0x02 {
                    panic!("unexpected control {:#04x} != 0x02", meta.control)
                }
                if meta.msg_type != 0x41 {
                    panic!("unexpected msg_type {:#04x} != 0x41", meta.msg_type)
                }

                res = Some(Association {
                    network_id: data.network_id,
                    association_id: meta.association_id,
                    request_id: meta.request_id,
                });
                println!("Sending association command:");
                send_cmd(
                    rf,
                    from,
                    0x80,
                    meta.association_id,
                    meta.request_id,
                    meta.control + 0x80,
                    meta.msg_type,
                    &AssociationCmd {
                        version: [0x01, 0x21, 0x01, 0x02],
                    }, // frisquet connect version
                )?;
            }
            None => {
                if res.is_some() {
                    break;
                }
            }
        }
    }

    match res {
        Some(v) => Ok(v),
        None => Err(ConnectError::new("association failed")),
    }
}

fn wait_association_msg(
    rf: &mut Box<dyn RFClient>,
    timeout: Duration,
) -> Result<Option<(Metadata, AssociationMsg)>, ConnectError> {
    loop {
        let payload = match rf.recv_timeout(timeout) {
            Ok(msg) => msg,
            Err(e) => {
                if e.is_timeout() {
                    return Ok(None);
                }
                return Err(e.into());
            }
        };

        let (meta, data) = from_bytes(&payload)?;
        println!("RECV {} {}", meta, data);

        return Ok(Some((meta, data)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_association_msg() {
        let payload = hex::decode("0b008012d402410412345678").unwrap();
        // let (_, payload) = dbg_dmp(parse_data, "data")(&payload.as_slice()).unwrap();
        let (meta, data) = from_bytes(&payload).unwrap();
        let x: AssociationMsg = data;

        assert_eq!(
            meta,
            Metadata {
                length: 11,
                to_addr: 0,
                from_addr: 128,
                association_id: 18,
                request_id: 212,
                control: 0x02,
                msg_type: 0x41,
            }
        );
        assert_eq!(
            x,
            AssociationMsg {
                len: 4,
                network_id: [0x12, 0x34, 0x56, 0x78],
            }
        );
    }

    #[test]
    fn test_association_reply() {
        let payload = hex::decode("0a80094914824101270002").unwrap();
        // let (_, payload) = dbg_dmp(parse_data, "data")(&payload.as_slice()).unwrap();
        let (meta, data) = from_bytes(&payload).unwrap();
        let x: AssociationCmd = data;

        assert_eq!(
            meta,
            Metadata {
                length: 10,
                to_addr: 128,
                from_addr: 0x09,
                association_id: 73,
                request_id: 20,
                control: 0x02 + 0x80,
                msg_type: 0x41,
            }
        );
        assert_eq!(
            x,
            AssociationCmd {
                version: [0x01, 0x27, 0x00, 0x02]
            }
        );
    }
}
