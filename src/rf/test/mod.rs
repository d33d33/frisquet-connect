use std::io::Error;
use std::time::{Duration, Instant};

use crate::rf::{RFClient, RecvError, RecvTimeoutError, SendError};

pub struct TestClient {
    replies: Vec<String>,
}

impl TestClient {
    fn try_recv(&mut self) -> Result<Option<Vec<u8>>, String> {
        return match self.replies.pop() {
            None => Err("No more data".to_string()),
            Some(m) => Ok(Some(hex::decode(m).unwrap())),
        };
    }
}

impl RFClient for TestClient {
    fn set_network_id(&mut self, network_id: Vec<u8>) -> Result<(), String> {
        println!("network_id : {}", hex::encode(network_id));
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
        loop {
            if let Some(data) = self.try_recv()? {
                return Ok(data);
            }
        }
    }

    fn recv_timeout(&mut self, timeout: Duration) -> Result<Vec<u8>, RecvTimeoutError> {
        let now = Instant::now();
        loop {
            if let Some(data) = self.try_recv()? {
                return Ok(data);
            }

            if now.elapsed() > timeout {
                return Err(RecvTimeoutError::Timeout);
            }
        }
    }

    fn send(&mut self, payload: Vec<u8>) -> Result<(), SendError> {
        println!("Send: {}", hex::encode(payload));
        Ok(())
    }

    fn sleep(&mut self) -> Result<(), String> {
        Ok(())
    }
}
