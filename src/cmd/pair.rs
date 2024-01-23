use crate::cmd::Entity;
use std::error::Error;

use crate::config::{self, Config};
use crate::connect::pair::connect_association;
use crate::rf::RFClient;

pub fn run(
    rf: &mut Box<dyn RFClient>,
    from: Entity,
    config: &mut Config,
) -> Result<(), Box<dyn Error>> {
    let fromAddr = match from {
        Entity::Connect => 0x7e,
        Entity::Sonde => 0x20,
        Entity::Satellite_Z1 => 0x08,
        Entity::Satellite_Z2 => 0x09,
        Entity::Satellite_Z3 => 0x0A,
    };
    let ass = connect_association(rf, fromAddr)?;

    println!(
        "SUCCESS: network_id: {:#?}, association_id: {:#?}",
        ass.network_id, ass.association_id
    );

    match from {
        Entity::Connect => {
            config.frisquet = Some(config::Frisquet {
                network_id: Some(ass.network_id),
                association_id: Some(ass.association_id),
                request_id: Some(ass.request_id),
                send_init: false,
            })
        }
        Entity::Sonde => {
            config.sonde = Some(config::Frisquet {
                network_id: Some(ass.network_id),
                association_id: Some(ass.association_id),
                request_id: Some(ass.request_id),
                send_init: true,
            })
        }
        Entity::Satellite_Z1 => (),
        Entity::Satellite_Z2 => (),
        Entity::Satellite_Z3 => (),
    };

    Ok(())
}
