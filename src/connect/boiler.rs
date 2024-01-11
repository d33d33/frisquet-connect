use colored::Colorize;
use deku::prelude::*;
use hex;
use std::fmt;

use super::Assert;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct BoilerMsg {
    cmd: [u8; 4],  // cmd
    cmd2: [u8; 4], // re cmd
    len: u8,       // length
    unknown: [u8; 42],
}

impl fmt::Display for BoilerMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes().map(hex::encode) {
            Ok(data) => {
                write!(
                    f,
                    "{}{}{}{}",
                    data[0..8].cyan(),
                    data[8..16].magenta(),
                    data[16..18].yellow(),
                    data[18..].white(),
                )?;

                write!(f, "\n    BoilerMsg")?;
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
                write!(f, "\n\t {}", format!("Length: {:0x}", self.len).yellow())
            }
            Err(_) => write!(f, "ERROR"),
        }
    }
}

impl Assert for BoilerMsg {
    fn assert(&self) -> bool {
        self.cmd == self.cmd2 && self.len == 0x2A
    }
}
