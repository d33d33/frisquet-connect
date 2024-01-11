use std::error::Error;

use crate::config::Config;
use crate::connect::area::connect_area1;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let config = &mut *config;
    let (_meta, _sensor) = connect_area1(rf, config)?;

    Ok(())
}
