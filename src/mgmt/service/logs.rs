use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

const MAX_BUFFER_LINES: usize = 500;
const BROADCAST_CAPACITY: usize = 256;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub timestamp: DateTime<Utc>,
    pub stream: LogStream,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogChannel {
    StorageBroker,
    StorageController,
    Pageserver,
    Safekeeper,
    StorageControllerDb,
    ManagementDb,
    ComputeEndpoint(Uuid),
    Pgbouncer(Uuid),
    Import(Uuid),
}

impl FromStr for LogChannel {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "storage_broker" => Ok(LogChannel::StorageBroker),
            "storage_controller" => Ok(LogChannel::StorageController),
            "pageserver" => Ok(LogChannel::Pageserver),
            "safekeeper" => Ok(LogChannel::Safekeeper),
            "storage_controller_db" => Ok(LogChannel::StorageControllerDb),
            "management_db" => Ok(LogChannel::ManagementDb),
            _ => Err(()),
        }
    }
}

struct LogBuffer {
    buffer: Mutex<VecDeque<LogLine>>,
    sender: broadcast::Sender<LogLine>,
}

impl LogBuffer {
    fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        LogBuffer {
            buffer: Mutex::new(VecDeque::new()),
            sender,
        }
    }

    fn push(&self, line: LogLine) {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() >= MAX_BUFFER_LINES {
            buffer.pop_front();
        }
        buffer.push_back(line.clone());
        drop(buffer);
        let _ = self.sender.send(line);
    }

    fn snapshot(&self) -> Vec<LogLine> {
        self.buffer.lock().unwrap().iter().cloned().collect()
    }

    fn subscribe(&self) -> broadcast::Receiver<LogLine> {
        self.sender.subscribe()
    }
}

pub struct LogsService {
    channels: Mutex<HashMap<LogChannel, Arc<LogBuffer>>>,
}

impl LogsService {
    pub fn new() -> Arc<Self> {
        Arc::new(LogsService {
            channels: Mutex::new(HashMap::new()),
        })
    }

    fn get_or_create(&self, channel: LogChannel) -> Arc<LogBuffer> {
        let mut channels = self.channels.lock().unwrap();
        channels
            .entry(channel)
            .or_insert_with(|| Arc::new(LogBuffer::new()))
            .clone()
    }

    pub fn ingest(&self, channel: LogChannel, message: String, stream: LogStream) {
        let line = LogLine {
            timestamp: Utc::now(),
            stream,
            message,
        };
        self.get_or_create(channel).push(line);
    }

    pub fn drop_channel(&self, channel: LogChannel) {
        self.channels.lock().unwrap().remove(&channel);
    }

    pub fn snapshot(&self, channel: LogChannel) -> Vec<LogLine> {
        let channels = self.channels.lock().unwrap();
        channels
            .get(&channel)
            .map(|buffer| buffer.snapshot())
            .unwrap_or_default()
    }

    pub fn subscribe(&self, channel: LogChannel) -> broadcast::Receiver<LogLine> {
        self.get_or_create(channel).subscribe()
    }
}