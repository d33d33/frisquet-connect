use colored::Colorize;
use deku::prelude::*;
use hex;
use std::fmt;

use crate::config;
use crate::connect::{ConnectError, Metadata};
use crate::rf::RFClient;

use super::Assert;

// a0f00015 a0f00015 2a e09065a100000000 321065a300000000 dd009a3c0000000000993233303435303939353330303131ffff
// a0f00015 a0f00015 2a 00000000000000000000000000000000  dd009a3c0000000000993233303435303939353330303131ffff
// a0f00015a0f00015  2a e09065a100000000321065a300000000  dd009a3c0000000000993233303435303939353330303131ffff

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct HolidayMsg {
    cmd: [u8; 4],  // cmd
    cmd2: [u8; 4], // re cmd
    len: u8,
    start: [u8; 8],
    end: [u8; 8],
    date1: [u8; 8],
    data: [u8; 18], // data
}

impl fmt::Display for HolidayMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes().map(hex::encode) {
            Ok(data) => {
                write!(
                    f,
                    "{}{}{}{}{}{}{}",
                    data[0..8].cyan(),
                    data[8..16].magenta(),
                    data[16..18].yellow(),
                    data[18..34].green(),
                    data[34..50].red(),
                    data[50..66].cyan(),
                    data[66..].white()
                )?;

                write!(f, "\n    HolidayMsg")?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Cmd:  {}", hex::encode(self.cmd)).cyan()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Cmd2: {}", hex::encode(self.cmd2)).magenta()
                )?;
                write!(f, "\n\t {}", format!("Length: {:0x}", self.len).yellow())?;
                write!(f, "\n\t {}", format!("Start: {}", (self.start())).red())?;
                write!(f, "\n\t {}", format!("End: {}", (self.end())).green())?;
                write!(f, "\n\t {}", format!("Date1: {}", (self.date1())).cyan())
            }
            Err(_) => write!(f, "ERROR"),
        }
    }
}

impl Assert for HolidayMsg {
    fn assert(&self) -> bool {
        self.cmd == self.cmd2 && self.len == 0x2A
    }
}

impl HolidayMsg {
    fn start(&self) -> chrono::NaiveDateTime {
        let secs: i64 = (((self.start[2] as u32) << 24)
            + ((self.start[3] as u32) << 16)
            + ((self.start[0] as u32) << 8)
            + self.start[1] as u32) as i64;
        chrono::NaiveDateTime::from_timestamp_opt(secs, 0).unwrap()
    }
    fn end(&self) -> chrono::NaiveDateTime {
        let secs: i64 = (((self.end[2] as u32) << 24)
            + ((self.end[3] as u32) << 16)
            + ((self.end[0] as u32) << 8)
            + self.end[1] as u32) as i64;
        chrono::NaiveDateTime::from_timestamp_opt(secs, 0).unwrap()
    }
    fn date1(&self) -> chrono::NaiveDateTime {
        let secs: i64 = (((self.date1[2] as u32) << 24)
            + ((self.date1[3] as u32) << 16)
            + ((self.date1[0] as u32) << 8)
            + self.date1[1] as u32) as i64;
        chrono::NaiveDateTime::from_timestamp_opt(secs, 0).unwrap()
    }
}

pub fn connect_holiday(
    _rf: &mut Box<dyn RFClient>,
    _config: &mut config::Config,
) -> Result<(Metadata, ()), ConnectError> {
    Err("unimplemented".to_string())?
}
