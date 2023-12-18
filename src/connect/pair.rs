use deku::prelude::*;
use hex;
use std::fmt;
use std::time::Duration;

use crate::connect::{from_bytes, send_cmd, ConnectError, Metadata};
use crate::rf::RFClient;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
struct AssociationCmd {
    version: [u8; 4],
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
struct AssociationMsg {
    unknown: u8,
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

pub fn connect_association(rf: &mut Box<dyn RFClient>) -> Result<Association, ConnectError> {
    let network_id: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff];
    rf.set_network_id(network_id)?;
    println!("Setting network_id to ffffffff"); // TODO log in set_network_id

    println!("Waiting broadcast message");
    let mut res: Option<Association> = None;
    loop {
        match wait_association_msg(rf, Duration::new(5, 0))? {
            Some((meta, data)) => {
                if meta.control != 0x02 {
                    panic!("unexpected control")
                }
                if meta.msg_type != 0x41 {
                    panic!("unexpected msg_type")
                }

                res = Some(Association {
                    network_id: data.network_id,
                    association_id: meta.association_id,
                    request_id: meta.request_id,
                });

                send_cmd(
                    rf,
                    0x7e,
                    0x80,
                    meta.association_id,
                    meta.request_id,
                    0x82,
                    meta.msg_type,
                    AssociationCmd {
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
