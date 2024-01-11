use std::error::Error;

use crate::config::Config;
use crate::connect::data1::connect_data1;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let (_meta, _date) = connect_data1(rf, config.frisquet()?)?;

    Ok(())
}
