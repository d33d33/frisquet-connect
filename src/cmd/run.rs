use chrono::Duration;
use std::error::Error;
use std::thread;

use crate::config::Config;
use crate::connect::sonde::{send_init, send_temperature};
use crate::datasource::externaltemperature::homeassistant;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let sonde_config = config.sonde()?;
    if sonde_config.send_init.unwrap_or(false) {
        let (_meta, _sensor) = send_init(rf, sonde_config)?;
        sonde_config.send_init = Some(false);
    }
    config.write()?;

    loop {
        let temperature = homeassistant::get_ha_client(config.home_assistant()?)?;
        println!("Set temperature to: {:.1}", temperature);
        let (_meta, _sensor) = send_temperature(rf, config.sonde()?, temperature)?;

        thread::sleep(Duration::minutes(3).to_std()?);
    }
}
