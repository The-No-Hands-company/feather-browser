// Web Workers - Multi-threading support
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug)]
pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: Sender<WorkerMessage>,
    next_id: usize,
}

#[derive(Debug)]
pub struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
    state: WorkerState,
}

#[derive(Debug, Clone, Copy)]
pub enum WorkerState {
    Idle,
    Running,
    Terminated,
}

#[derive(Debug, Clone)]
pub enum WorkerMessage {
    Execute { id: usize, script: String },
    Terminate { id: usize },
    PostMessage { from: usize, to: usize, data: WorkerData },
}

#[derive(Debug, Clone)]
pub enum WorkerData {
    String(String),
    Number(f64),
    Object(Vec<(String, WorkerData)>),
    Array(Vec<WorkerData>),
    Binary(Vec<u8>),
}

impl WorkerPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::new();
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender,
            next_id: size,
        }
    }

    pub fn spawn_worker(&mut self, script: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let (sender, receiver) = channel();
        let receiver_clone = Arc::new(Mutex::new(receiver));
        
        let worker = Worker::new(id, receiver_clone);
        self.workers.push(worker);

        // Store sender for future communication
        if let Err(e) = self.sender.send(WorkerMessage::Execute {
            id,
            script,
        }) {
            log::error!("Failed to send worker message: {}", e);
        }

        // TODO: Store sender in worker for bidirectional communication
        drop(sender);

        id
    }

    pub fn post_message(&self, worker_id: usize, data: WorkerData) {
        self.sender.send(WorkerMessage::PostMessage {
            from: 0,
            to: worker_id,
            data,
        }).ok();
    }

    pub fn terminate_worker(&mut self, worker_id: usize) {
        self.sender.send(WorkerMessage::Terminate { id: worker_id }).ok();
        
        if let Some(worker) = self.workers.iter_mut().find(|w| w.id == worker_id) {
            worker.state = WorkerState::Terminated;
            if let Some(handle) = worker.handle.take() {
                handle.join().ok();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<WorkerMessage>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                let message = {
                    let rx = receiver.lock().unwrap();
                    rx.recv()
                };

                match message {
                    Ok(WorkerMessage::Execute { id: msg_id, script }) => {
                        if msg_id == id {
                            // Execute script (simplified)
                            println!("Worker {} executing: {}", id, script);
                        }
                    }
                    Ok(WorkerMessage::PostMessage { to, data, .. }) => {
                        if to == id {
                            println!("Worker {} received message: {:?}", id, data);
                        }
                    }
                    Ok(WorkerMessage::Terminate { id: msg_id }) => {
                        if msg_id == id {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            id,
            handle: Some(handle),
            state: WorkerState::Idle,
        }
    }
}

// Shared Array Buffer for worker communication
#[derive(Debug)]
pub struct SharedArrayBuffer {
    data: Arc<Mutex<Vec<u8>>>,
    byte_length: usize,
}

impl SharedArrayBuffer {
    pub fn new(byte_length: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(vec![0u8; byte_length])),
            byte_length,
        }
    }

    pub fn get(&self, index: usize) -> Option<u8> {
        let data = self.data.lock().unwrap();
        data.get(index).copied()
    }

    pub fn set(&self, index: usize, value: u8) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        if index < data.len() {
            data[index] = value;
            Ok(())
        } else {
            Err("Index out of bounds".to_string())
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> Option<Vec<u8>> {
        let data = self.data.lock().unwrap();
        if end <= data.len() {
            Some(data[start..end].to_vec())
        } else {
            None
        }
    }

    pub fn clone_buffer(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            byte_length: self.byte_length,
        }
    }
}

// Atomics for thread-safe operations
pub struct Atomics;

impl Atomics {
    pub fn load(buffer: &SharedArrayBuffer, index: usize) -> Option<u8> {
        buffer.get(index)
    }

    pub fn store(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Result<(), String> {
        buffer.set(index, value)
    }

    pub fn add(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Result<u8, String> {
        let mut data = buffer.data.lock().unwrap();
        if index < data.len() {
            let old = data[index];
            data[index] = old.wrapping_add(value);
            Ok(old)
        } else {
            Err("Index out of bounds".to_string())
        }
    }

    pub fn compare_exchange(
        buffer: &SharedArrayBuffer,
        index: usize,
        expected: u8,
        replacement: u8
    ) -> Result<u8, String> {
        let mut data = buffer.data.lock().unwrap();
        if index < data.len() {
            let old = data[index];
            if old == expected {
                data[index] = replacement;
            }
            Ok(old)
        } else {
            Err("Index out of bounds".to_string())
        }
    }

    pub fn wait(_buffer: &SharedArrayBuffer, _index: usize, _value: u8, _timeout: Option<u64>) 
        -> WaitResult {
        // Simplified - would implement futex-like waiting
        WaitResult::Ok
    }

    pub fn notify(_buffer: &SharedArrayBuffer, _index: usize, _count: usize) -> usize {
        // Simplified - would wake waiting threads
        0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WaitResult {
    Ok,
    NotEqual,
    TimedOut,
}
