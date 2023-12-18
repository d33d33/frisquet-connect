use std::error::Error;

use crate::config::Config;
use crate::connect::sensors::connect_sensors;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let (_meta, _sensor) = connect_sensors(rf, config.frisquet()?)?;

    Ok(())
}
