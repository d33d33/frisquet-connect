use std::error::Error;

use crate::config::Config;
use crate::connect::date::connect_date;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let (_meta, _date) = connect_date(rf, config.frisquet()?)?;

    Ok(())
}
