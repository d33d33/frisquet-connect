use hex;
use mqtt::{Message, Receiver};
use paho_mqtt as mqtt;
use std::result::Result;
use std::time::Duration;

use crate::config;
use crate::rf::mqtt::messages::{CommandMessage, Listen, SendData, SetNetworkId, Sleep};
use crate::rf::{RFClient, RecvError, RecvTimeoutError, SendError};

pub mod messages;

pub struct MqttClient {
    client: mqtt::Client,
    rx: Receiver<Option<Message>>,
    cmd_topic: String,
}

pub fn new(config: &config::MQTT) -> Result<MqttClient, String> {
    // Define the set of options for the create.
    // Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(&config.broker)
        .client_id(&config.client_id)
        .finalize();

    // Create a client.
    let client = mqtt::Client::new(create_opts).map_err(|e| e.to_string())?;
    let rx: Receiver<Option<Message>> = client.start_consuming();
    // Define the set of options for the connection.
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = client.connect(conn_opts) {
        return Err(format!("Unable to connect:\n\t{:?}", e));
    }

    if let Err(e) = client.subscribe(&config.lst_topic, 0) {
        return Err(format!("Error subscribes topics: {:?}", e));
    }

    Ok(MqttClient {
        client: client,
        rx: rx,
        cmd_topic: config.cmd_topic.clone(),
    })
}

impl MqttClient {
    fn publish(&self, value: &dyn CommandMessage) -> Result<(), String> {
        let json = serde_json::to_vec(value).map_err(|e| e.to_string())?;

        if let Err(e) = self.client.publish(Message::new(&self.cmd_topic, json, 0)) {
            return Err(format!("Error subscribes topics: {:?}", e));
        }
        Ok(())
    }
}

impl RFClient for MqttClient {
    fn set_network_id(&mut self, network_id: Vec<u8>) -> Result<(), String> {
        self.publish(&SetNetworkId {
            network_id: hex::encode(network_id),
        })
    }

    fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
        self.publish(&Listen {})?;
        let msg = self.rx.recv().map_err(|e| e.to_string())?;
        match msg {
            Some(msg) => {
                let data: messages::DataMessage =
                    serde_json::from_str(msg.payload_str().as_ref()).map_err(|e| e.to_string())?;
                hex::decode(data.data).map_err(|e| RecvError { msg: e.to_string() })
            }
            None => self.recv(),
        }
    }

    fn recv_timeout(&mut self, timeout: Duration) -> Result<Vec<u8>, RecvTimeoutError> {
        self.publish(&Listen {})?;

        let msg = self.rx.recv_timeout(timeout).map_err(|e| {
            if e.is_timeout() {
                RecvTimeoutError::Timeout
            } else {
                RecvTimeoutError::Error { msg: e.to_string() }
            }
        })?;

        match msg {
            Some(msg) => {
                let data: messages::DataMessage =
                    serde_json::from_str(msg.payload_str().as_ref()).map_err(|e| e.to_string())?;
                hex::decode(data.data).map_err(|e| RecvTimeoutError::Error { msg: e.to_string() })
            }
            None => Err(RecvTimeoutError::Timeout),
        }
    }

    fn send(&mut self, payload: Vec<u8>) -> Result<(), SendError> {
        self.publish(&SendData {
            payload: hex::encode(payload),
        })
        .map_err(|e| SendError { msg: e.to_string() })
    }

    fn sleep(&mut self) -> Result<(), String> {
        self.publish(&Sleep {})
    }
}
