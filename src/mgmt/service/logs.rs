use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

const MAX_BUFFER_BYTES: usize = 1024 * 1024;
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

struct LogBufferState {
    lines: VecDeque<LogLine>,
    bytes: usize,
}

struct LogBuffer {
    buffer: Mutex<LogBufferState>,
    sender: broadcast::Sender<LogLine>,
}

impl LogBuffer {
    fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        LogBuffer {
            buffer: Mutex::new(LogBufferState {
                lines: VecDeque::new(),
                bytes: 0,
            }),
            sender,
        }
    }

    fn push(&self, line: LogLine) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.bytes += line.message.len();
        buffer.lines.push_back(line.clone());
        while buffer.bytes > MAX_BUFFER_BYTES {
            match buffer.lines.pop_front() {
                Some(removed) => buffer.bytes -= removed.message.len(),
                None => break,
            }
        }
        drop(buffer);
        let _ = self.sender.send(line);
    }

    fn snapshot(&self) -> Vec<LogLine> {
        self.buffer.lock().unwrap().lines.iter().cloned().collect()
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