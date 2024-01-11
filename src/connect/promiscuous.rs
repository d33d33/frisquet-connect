use colored::Colorize;
use deku::prelude::*;
use hex;
use std::collections::HashMap;

use crate::config;
use crate::connect::{area, boiler, from_bytes, sensors, ConnectError, Metadata};
use crate::rf::RFClient;

use super::date;

pub fn connect_promiscuous(
    rf: &mut Box<dyn RFClient>,
    config: &mut config::Frisquet,
) -> Result<(), ConnectError> {
    rf.set_network_id(Vec::from(config.network_id()?))?;

    let mut inflight: HashMap<u8, String> = HashMap::new();

    loop {
        let payload = &rf.recv()?;
        let meta = match Metadata::from_bytes((payload, 0)) {
            Ok((_, meta)) => meta,
            Err(e) => {
                println!("meta err: {}", e.to_string());
                continue;
            }
        };

        if meta.from_addr != 0x7e && meta.to_addr != 0x7e {
            continue;
        }

        match inflight.get(&meta.request_id) {
            None => match meta.msg_type {
                0x17 => {
                    println!("{:#?}", hex::encode(payload.split_at(4).0));
                    match payload[7..11] {
                        [0xa1, 0x54, 0x00, 0x18] => {
                            let data = match from_bytes::<area::AreaMsg>(payload) {
                                Ok((_, data)) => data,
                                Err(e) => {
                                    println!(
                                        "ProgMsg err: {} - {}",
                                        e.to_string(),
                                        hex::encode(&payload[7..])
                                    );
                                    continue;
                                }
                            };
                            println!("=> {} {}", meta, data);
                            inflight.insert(meta.request_id, "a1540018".into());
                        }
                        [0xa0, 0xf0, 0x00, 0x15] => {
                            let data = match from_bytes::<boiler::BoilerMsg>(payload) {
                                Ok((_, data)) => data,
                                Err(e) => {
                                    println!(
                                        "ProgMsg err: {} - {}",
                                        e.to_string(),
                                        hex::encode(&payload[7..])
                                    );
                                    continue;
                                }
                            };
                            println!("=> {} {}", meta, data);
                            inflight.insert(meta.request_id, "a0f00015".into());
                        }
                        _ => {
                            let data = hex::encode(&payload[7..]);
                            println!("=> {} {}", meta, data);
                            inflight.insert(meta.request_id, "17".into());
                        }
                    }
                }
                _ => {
                    let data = hex::encode(&payload[7..]);
                    println!("=> {} {}", meta, data);
                    inflight.insert(meta.request_id, data);
                }
            },
            Some(req) => {
                match req.as_str() {
                    "a02b0004" => {
                        let data = match from_bytes::<date::DateMsg>(payload) {
                            Ok((_, data)) => data,
                            Err(e) => {
                                println!(
                                    "DateMsg err: {} - {}",
                                    e.to_string(),
                                    hex::encode(&payload[7..])
                                );
                                continue;
                            }
                        };
                        println!("<= {} {}", meta, data);
                    }
                    "79e0001c" => {
                        let data = match from_bytes::<sensors::SensorsMsg>(payload) {
                            Ok((_, data)) => data,
                            Err(e) => {
                                println!(
                                    "SensorsMsg err: {} - {}",
                                    e.to_string(),
                                    hex::encode(&payload[7..])
                                );
                                continue;
                            }
                        };
                        println!("<= {} {}", meta, data);
                    }
                    "a1540018" | "a0f00015" | "17" => {
                        let data = hex::encode(&payload[7..]);
                        println!("<= {} {}", meta, data);
                    }
                    cmd => {
                        println!("UNKNOWN cmd: {}", cmd.red());
                        let data = hex::encode(&payload[7..]);
                        println!("<= {} {}", meta, data);
                    }
                };
                inflight.remove(&meta.request_id);
            }
        }

        // let (_, data) = T::from_bytes((&payload[7..], 0))?;
        // match filter(&rf.recv()?, 0x80, 0x7e, config.association_id()?, req_id)? {
        //     Some(payload) => {
        //         let (meta, data) = from_bytes(&payload)?;
        //         println!("RECV {} {}", meta, data);
        //         return Ok((meta, data));
        //     }
        //     None => {}
        // }
    }
}
