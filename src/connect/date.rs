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
pub struct DateMsg {
    len: u8,
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    data: u8,
    weekday: u8,
}

impl fmt::Display for DateMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes().map(hex::encode) {
            Ok(data) => {
                write!(
                    f,
                    "{}{}{}{}{}{}{}{}{}",
                    data[0..2].white(),
                    data[2..4].yellow(),
                    data[4..6].purple(),
                    data[6..8].blue(),
                    data[8..10].red(),
                    data[10..12].green(),
                    data[12..14].magenta(),
                    data[14..16].white(),
                    data[16..].yellow(),
                )?;

                write!(f, "\n    DateMsg")?;
                write!(f, "\n\t {}", format!("Year: {:x}", self.year).yellow())?;
                write!(f, "\n\t {}", format!("Month: {:x}", self.month).purple())?;
                write!(f, "\n\t {}", format!("Day: {:x}", self.day).blue())?;
                write!(f, "\n\t {}", format!("Hour: {:x}", self.hour).red())?;
                write!(f, "\n\t {}", format!("Minute: {:x}", self.minute).green())?;
                write!(f, "\n\t {}", format!("Second: {:x}", self.second).magenta())?;
                write!(f, "\n\t {}", format!("Weekday: {}", self.weekday).yellow())
            }
            Err(_) => write!(f, "ERROR"),
        }
    }
}

impl Assert for DateMsg {
    fn assert(&self) -> bool {
        self.len == 8 // length is expected to the payload length(8)
    }
}

impl DateMsg {
    pub fn year(&self) -> u8 {
        (self.year >> 4) * 10 + (self.year & 0x0F)
    }
    pub fn month(&self) -> u8 {
        (self.month >> 4) * 10 + (self.month & 0x0F)
    }
    pub fn day(&self) -> u8 {
        (self.day >> 4) * 10 + (self.day & 0x0F)
    }
    pub fn hour(&self) -> u8 {
        (self.hour >> 4) * 10 + (self.hour & 0x0F)
    }
    pub fn minute(&self) -> u8 {
        (self.minute >> 4) * 10 + (self.hour & 0x0F)
    }
    pub fn second(&self) -> u8 {
        (self.second >> 4) * 10 + (self.second & 0x0F)
    }
    pub fn weekday(&self) -> u8 {
        self.weekday
    }
}

pub fn connect_date(
    rf: &mut Box<dyn RFClient>,
    config: &mut config::Frisquet,
) -> Result<(Metadata, DateMsg), ConnectError> {
    rf.set_network_id(Vec::from(config.network_id()?))?;

    let req_id = config.next_req_id()?;
    // a02b0004
    send_cmd(
        rf,
        0x7e, // from
        0x80, // to
        config.association_id()?,
        req_id,
        1,
        03,
        &Cmd {
            data: vec![0xa0, 0x2b, 0x00, 0x04],
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
