use colored::Colorize;
use deku::prelude::*;
use hex;
use std::fmt;
use std::time::Duration;

use crate::config;
use crate::connect::date::connect_date;
use crate::connect::{filter, format_day, from_bytes, send_cmd, ConnectError, DropMsg, Metadata};
use crate::rf::RFClient;

use super::Assert;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct AreaMsg {
    cmd: [u8; 4],     // cmd
    cmd2: [u8; 4],    // re cmd
    len: u8,          // length
    temp_comfort: u8, // start at 5°C - 0 is 50
    temp_reduced: u8, // start at 5°C - 0 is 50
    temp_frost: u8,   // start at 5°C - 0 is 50
    mode: u8,         // 05 auto - 06 confort - 07 reduit - 08 hors gel
    #[deku(bits = "1")]
    unknow_mode: bool,
    #[deku(bits = "1")]
    boost: bool,
    #[deku(bits = "2")]
    unknown_mode2: u8,
    #[deku(bits = "2")]
    unknown_mode3: u8,
    #[deku(bits = "1")]
    derogation: bool,
    #[deku(bits = "1")]
    confort: bool,
    unknown_data: u8,
    sunday: [u8; 6],
    monday: [u8; 6],
    tuesday: [u8; 6],
    wednesday: [u8; 6],
    thursday: [u8; 6],
    friday: [u8; 6],
    saturday: [u8; 6],
}

impl fmt::Display for AreaMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes().map(hex::encode) {
            Ok(data) => {
                write!(
                    f,
                    "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                    data[0..8].cyan(),
                    data[8..16].magenta(),
                    data[16..18].yellow(),
                    data[18..20].red(),
                    data[20..22].green(),
                    data[22..24].purple(),
                    data[24..26].yellow(),
                    data[26..28].cyan(),
                    data[28..30].white(),
                    data[30..42].green(),
                    data[42..54].yellow(),
                    data[54..66].green(),
                    data[66..78].yellow(),
                    data[78..90].green(),
                    data[90..102].yellow(),
                    data[102..].green(),
                )?;

                // derogation, permanent, vacances
                // 0621 confort zone1
                // 0521 auto zone1
                // 0721 reduit zone1
                // 0500 auto zone 1
                // 0821 hors gel zone 1
                // 0511 auto

                // 05 selecteur auto - 06 selecteur confort - 07 selecteur reduit - 08 selecteur hors gel

                // 23 derog reduit
                // 20 annul derog reduit
                // 26 derog confort
                // 25 annul derog confort
                // 61 boost
                // 63 derog reduit + boost

                // 23 derog confort
                // 20 no derog reduit
                // 21 no derig confort
                // 22 derog reduit

                // entete
                // a1 540018a154001830 zone 1
                // f9 540018a154001830 ????
                // a1 540018a154001830

                let m = match self.mode {
                    0x05 => "auto",
                    0x06 => "confort",
                    0x07 => "reduit",
                    0x08 => "hors gel",
                    _ => "unknown",
                };

                write!(f, "\n    AreaMsg")?;
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
                write!(
                    f,
                    "\n\t {}",
                    format!("Confort T: {}", (self.temp_comfort + 50)).red()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Reduit T: {}", (self.temp_reduced + 50)).green()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Hors gel T: {}", (self.temp_frost + 50)).purple()
                )?;
                write!(f, "\n\t {}", format!("Mode: {m}").yellow())?;
                write!(f, "\n\t {}", format!("Boost: {}", self.boost).cyan())?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Derogation: {}", self.derogation).cyan()
                )?;
                write!(f, "\n\t {}", format!("Confort: {}", self.confort).cyan())?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Sunday:    {}", format_day(self.sunday)).green()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Monday:    {}", format_day(self.monday)).yellow()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Tuesday:   {}", format_day(self.tuesday)).green()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Wednesday: {}", format_day(self.wednesday)).yellow()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Thursday:  {}", format_day(self.thursday)).green()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Friday:    {}", format_day(self.friday)).yellow()
                )?;
                write!(
                    f,
                    "\n\t {}",
                    format!("Saturday:  {}", format_day(self.saturday)).green()
                )
            }
            Err(_) => write!(f, "ERROR"),
        }
    }
}

impl Assert for AreaMsg {
    fn assert(&self) -> bool {
        self.cmd == self.cmd2 && self.len == 0x30
    }
}

pub fn connect_area1(
    rf: &mut Box<dyn RFClient>,
    config: &mut config::Config,
) -> Result<(Metadata, ()), ConnectError> {
    rf.set_network_id(Vec::from(config.frisquet()?.network_id()?))?;

    let (_meta, date) = connect_date(rf, config.frisquet()?)?;

    let mut req_id = config.frisquet()?.next_req_id()?;
    let prog = config.area1()?;

    let curr_mode = if prog.mode_is_auto()? {
        let slot = date.hour() * 2 + date.minute() / 30;
        match date.weekday() {
            1 => prog.monday()?[(slot / 8) as usize] & (1 << slot % 8) != 0,
            2 => prog.tuesday()?[(slot / 8) as usize] & (1 << slot % 8) != 0,
            3 => prog.wednesday()?[(slot / 8) as usize] & (1 << slot % 8) != 0,
            4 => prog.thursday()?[(slot / 8) as usize] & (1 << slot % 8) != 0,
            5 => prog.friday()?[(slot / 8) as usize] & (1 << slot % 8) != 0,
            6 => prog.saturday()?[(slot / 8) as usize] & (1 << slot % 8) != 0,
            7 => prog.sunday()?[(slot / 8) as usize] & (1 << slot % 8) != 0,
            _ => Err(String::from("invalid weekday"))?,
        }
    } else {
        prog.mode_is_comfort()?
    };

    let msg = AreaMsg {
        cmd: [0xa1, 0x54, 0x00, 0x18],
        cmd2: [0xa1, 0x54, 0x00, 0x18],
        len: 0x30,
        temp_comfort: prog.comfort()?,
        temp_reduced: prog.reduced()?,
        temp_frost: prog.frost()?,
        mode: prog.mode()?,
        unknow_mode: false,
        boost: prog.boost,
        unknown_mode2: 0,
        unknown_mode3: 1,
        derogation: prog.comfort_override()?.is_some(),
        confort: prog.comfort_override()?.unwrap_or(curr_mode),
        unknown_data: 0,
        sunday: prog.sunday()?,
        monday: prog.monday()?,
        tuesday: prog.tuesday()?,
        wednesday: prog.wednesday()?,
        thursday: prog.thursday()?,
        friday: prog.friday()?,
        saturday: prog.saturday()?,
    };

    println!("{}", msg);

    let mut retry = 0;
    loop {
        send_cmd(
            rf,
            0x7e, // from
            0x80, // to
            config.frisquet()?.association_id()?,
            req_id,
            0x08,
            0x17,
            &msg,
        )?;

        match wait_response(
            rf,
            config.frisquet()?.association_id()?,
            req_id,
            Duration::new(5, 0),
        )? {
            Some((meta, _)) => return Ok((meta, ())),
            None => {
                retry += 1;
                req_id += 1;
                if retry == 3 {
                    retry = 0;
                    req_id = config.frisquet()?.next_req_id()?;
                }
            }
        }
    }
}

fn wait_response(
    rf: &mut Box<dyn RFClient>,
    association_id: u8,
    req_id: u8,
    timeout: Duration,
) -> Result<Option<(Metadata, ())>, ConnectError> {
    loop {
        let payload = match rf.recv_timeout(timeout) {
            Ok(msg) => msg,
            Err(e) => {
                if !e.is_timeout() {
                    return Err(e.into());
                }
                return Ok(None);
            }
        };
        match filter(&payload, 0x80, 0x7e, association_id, req_id)? {
            Some(payload) => {
                let (meta, data) = from_bytes::<DropMsg>(&payload)?;
                println!("RECV {} {}", meta, data);
                return Ok(Some((meta, ())));
            }
            None => {}
        }
    }
}
