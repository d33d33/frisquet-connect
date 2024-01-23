use std::error::Error;

use crate::config::Config;
use crate::connect::sonde::{send_init, send_temperature};
use crate::rf::RFClient;


pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let sonde_config = config.sonde()?;
    if sonde_config.send_init {
        let (_meta, _sensor) = send_init(rf, sonde_config)?;
        sonde_config.send_init = false;
    }


    let (_meta, _sensor) = send_temperature(rf, sonde_config, 130)?;

    Ok(())
}
