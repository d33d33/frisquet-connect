use std::error::Error;

use crate::config::{self, Config};
use crate::connect::pair::connect_association;
use crate::rf::RFClient;

pub fn run(rf: &mut Box<dyn RFClient>, config: &mut Config) -> Result<(), Box<dyn Error>> {
    let ass = connect_association(rf)?;

    println!(
        "SUCCESS: network_id: {:#?}, association_id: {:#?}",
        ass.network_id, ass.association_id
    );

    config.frisquet = Some(config::Frisquet {
        network_id: Some(ass.network_id),
        association_id: Some(ass.association_id),
        request_id: Some(ass.request_id),
    });

    Ok(())
}
