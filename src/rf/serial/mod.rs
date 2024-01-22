use hex;
use serialport;
use std::collections::VecDeque;
use std::result::Result;
use std::time::{Duration, Instant};

use crate::config;
use crate::rf::{RFClient, RecvError, RecvTimeoutError, SendError};

pub struct SerialClient {
    port: Box<dyn serialport::SerialPort>,
    buffer: Vec<u8>,
    data_packets: VecDeque<Vec<u8>>,
    mode: Mode,
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Mode {
    Idle,
    Listen,
    Sleep,
}

pub fn new(config: &config::Serial) -> Result<SerialClient, String> {
    let port = serialport::new(&config.port, config.speed)
        .timeout(Duration::from_millis(10))
        .open()
        .map_err(|err| format!("failed to open serial port: {}", err.to_string()))?;

    Ok(SerialClient {
        port: port,
        buffer: vec![],
        data_packets: VecDeque::new(),
        mode: Mode::Idle,
    })
}

impl SerialClient {
    fn try_recv(&mut self) -> Result<Option<Vec<u8>>, String> {
        if let Some(data) = self.data_packets.pop_front() {
            return Ok(Some(data));
        }

        if self.mode != Mode::Listen {
            let cmd = format!("LST:\n");
            self.port
                .write_all(cmd.as_bytes())
                .map_err(|e| e.to_string())?;
            self.port.flush().map_err(|e| e.to_string())?;
            self.mode = Mode::Listen
        }

        let mut buf = [0; 512];
        let read = match self.port.read(&mut buf) {
            Ok(v) => Ok(v),
            Err(e) => match e.kind() {
                std::io::ErrorKind::TimedOut => Ok(0 as usize),
                error => Err(error.to_string()),
            },
        }?;

        for n in 0..read {
            if buf[n] == 0xd {
                // \r
                continue;
            }
            if buf[n] == 0xA {
                // \r
                let data = hex::decode(&self.buffer)
                    // .map_err(|e| {
                    //     println!("{}: {}", e, String::from_utf8(self.buffer.clone()).unwrap())
                    // })
                    .unwrap_or(vec![]);

                if !data.is_empty() {
                    self.data_packets.push_back(data)
                }
                self.buffer.clear();
            } else {
                self.buffer.push(buf[n]);
            }
        }

        Ok(self.data_packets.pop_front())
    }
}

impl RFClient for SerialClient {
    fn set_network_id(&mut self, network_id: Vec<u8>) -> Result<(), String> {
        self.mode = Mode::Idle;

        let cmd = format!("NID: {}\n", hex::encode(network_id));
        self.port
            .write_all(cmd.as_bytes())
            .map(|_x| ())
            .map_err(|e| e.to_string())?;
        self.port.flush().map_err(|e| e.to_string())
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
                println!("received : {}", hex::encode(data.clone()));
                return Ok(data);
            }

            if now.elapsed() > timeout {
                return Err(RecvTimeoutError::Timeout);
            }
        }
    }

    fn send(&mut self, payload: Vec<u8>) -> Result<(), SendError> {
        self.mode = Mode::Idle;

        let cmd = format!("CMD: {}\n", hex::encode(payload));
        self.port
            .write_all(cmd.as_bytes())
            .map(|_x| ())
            .map_err(|e| e.to_string())?;
        self.port
            .flush()
            .map_err(|e| SendError { msg: e.to_string() })
    }

    fn sleep(&mut self) -> Result<(), String> {
        self.mode = Mode::Sleep;

        let cmd = format!("SLP:\n");
        self.port
            .write_all(cmd.as_bytes())
            .map(|_x| ())
            .map_err(|e| e.to_string())?;
        self.port.flush().map_err(|e| e.to_string())
    }
}
