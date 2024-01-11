use colored::Colorize;
use deku::prelude::*;
use hex;
use std::fmt;

use crate::config;
use crate::connect::{filter, from_bytes, send_cmd, Cmd, ConnectError, Metadata};
use crate::rf::RFClient;

use super::Assert;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct SensorsMsg {
    len: u8,
    temp_ecs: i16,
    temp_cdc: i16,
    temp_depart_1: i16,
    temp_depart_2: i16,
    temp_depart_3: i16,
    data: [u8; 26],
    temp_ambi_1: i16,
    temp_ambi_2: i16,
    temp_ambi_3: i16,
    data2: [u8; 6],
    temp_cons_1: i16,
    temp_cons_2: i16,
    temp_cons_3: i16,
    temp_exterieur: i16,
}

impl fmt::Display for SensorsMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes().map(hex::encode) {
            Ok(data) => {
                write!(
                    f,
                    "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                    data[0..2].white(),
                    data[2..6].yellow(),
                    data[6..10].purple(),
                    data[10..14].blue(),
                    data[14..18].red(),
                    data[18..22].green(),
                    data[22..74].white(),
                    data[74..78].magenta(),
                    data[78..82].yellow(),
                    data[82..86].blue(),
                    data[86..98].white(),
                    data[98..102].green(),
                    data[102..106].red(),
                    data[106..110].blue(),
                    data[110..114].magenta(),
                )?;

                write!(f, "\n    SensorsMessage")?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp ECS: {}", self.temp_ecs).yellow()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp CDC: {}", self.temp_cdc).purple()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Depart 1: {}", self.temp_depart_1).blue()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Depart 2: {}", self.temp_depart_2).red()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Depart 3: {}", self.temp_depart_3).green()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Ambi 1: {}", self.temp_ambi_1).magenta()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Ambi 2: {}", self.temp_ambi_2).yellow()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Ambi 3: {}", self.temp_ambi_3).blue()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Cons 1: {}", self.temp_cons_1).green()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Cons 2: {}", self.temp_cons_2).red()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Cons 3: {}", self.temp_cons_3).blue()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Temp Exterireur: {}", self.temp_exterieur).magenta()
                )
            }
            Err(_) => write!(f, "ERROR"),
        }
    }
}

impl Assert for SensorsMsg {
    fn assert(&self) -> bool {
        self.len as usize == 0x38 // length is expected to represent the msg length(56)
    }
}

pub fn connect_sensors(
    rf: &mut Box<dyn RFClient>,
    config: &mut config::Frisquet,
) -> Result<(Metadata, SensorsMsg), ConnectError> {
    rf.set_network_id(Vec::from(config.network_id()?))?;

    let req_id = config.next_req_id()?;
    // 79e0001c
    send_cmd(
        rf,
        0x7e, // from
        0x80, // to
        config.association_id()?,
        req_id,
        1,
        03,
        &Cmd {
            data: vec![0x79, 0xe0, 0x00, 0x1c],
        },
    )?;

    loop {
        match filter(&rf.recv()?, 0x80, 0x7e, config.association_id()?, req_id)? {
            Some(payload) => {
                let (meta, data) = from_bytes(&payload)?;
                println!("RECV {} {}", meta, data);
                return Ok((meta, data));
            }
            None => {}
        }
    }
}
