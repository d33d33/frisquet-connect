use std::error::Error;

use crate::config::Config;
use crate::connect::sonde::send_temperature;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let (_meta, _sensor) = send_temperature(rf, config.sonde()?, 130)?;

    Ok(())
}
