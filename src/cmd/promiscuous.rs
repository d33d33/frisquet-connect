use std::error::Error;

use crate::config::Config;
use crate::connect::promiscuous::connect_promiscuous;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    connect_promiscuous(rf, config.frisquet()?)?;

    Ok(())
}
