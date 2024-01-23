use crate::config;
use crate::datasource::externaltemperature::ExternalTemperatureErr;
use serde_json::Value;
use std::collections::HashMap;

impl From<reqwest::Error> for ExternalTemperatureErr {
    fn from(value: reqwest::Error) -> Self {
        ExternalTemperatureErr::from(value.to_string())
    }
}

pub fn get_ha_client(config: &mut config::HAConfig) -> Result<f32, ExternalTemperatureErr> {
    let ha_state_url = format!("{}/api/states/{}", config.host.as_str(), config.entity_id);

    // request::blocking => Client.get().header(xxx, config.token)
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(ha_state_url)
        .header("Authorization", format!("Bearer {}", config.token))
        .send()?;
    // println!("response: {}", response.text());
    let response_json = response.json::<HashMap<String, Value>>()?;

    let attribute = response_json
        .get("attributes")
        .ok_or(ExternalTemperatureErr::from(format!(
            "Unknown field {}",
            config.temperature_field
        )))?;

    let temp_str =
        attribute
            .get(config.temperature_field.as_str())
            .ok_or(ExternalTemperatureErr::from(format!(
                "Unknown field {}",
                config.temperature_field
            )))?;

    return temp_str
        .to_string()
        .parse::<f32>()
        .map_err(|e| ExternalTemperatureErr::from(format!("Cannot parse: {}", e.to_string())));
}
