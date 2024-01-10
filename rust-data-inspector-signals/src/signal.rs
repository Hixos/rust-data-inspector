use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Receiver, SendError, Sender};

use regex::Regex;
use thiserror::Error;

pub struct Signal {
    id: SignalID,
    name: String,

    time: Vec<f64>,
    data: Vec<f64>,
}

impl Signal {
    pub fn new(name: String, id: SignalID) -> Self {
        Signal {
            id,
            name,
            time: vec![],
            data: vec![],
        }
    }

    pub fn id(&self) -> SignalID {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path_elements(&self) -> Vec<&str> {
        self.name.split('/').collect::<Vec<&str>>()
    }

    pub fn time(&self) -> &Vec<f64> {
        &self.time
    }

    pub fn data(&self) -> &Vec<f64> {
        &self.data
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SignalID {
    id: u64,
}

#[derive(Clone, Copy)]
pub struct SignalSample {
    pub time: f64,
    pub value: f64,
}

#[derive(Default)]
pub struct Signals {
    signals: HashMap<SignalID, Signal>,
    receivers: HashMap<SignalID, Receiver<SignalSample>>,
}

impl Signals {
    pub fn get_signals(&self) -> &HashMap<SignalID, Signal> {
        &self.signals
    }

    /// Returns the signal with the provided id.  
    /// Assumes that the SignalID was provided a call to add_signal of this instance.  
    /// Panics if a signal with the provided id is not found. This happens if the id belongs to another instance.
    pub fn get_signal(&self, id: SignalID) -> &Signal {
        self.signals.get(&id).unwrap()
    }

    /// Creates a new signal, returning its ID and the associated sample producer  
    ///   
    /// Valid signal names must start with `/`, and may be divided in many parts separated by additional `/`s, just like unix paths.  
    /// No two consecutive `/`s may be present. Each part may only include letters, numbers and underscores '_'.
    /// 
    /// It is illegal for a signal to be a sub-signal of a previously added one.
    /// 
    /// ## Examples:
    /// - `/status`  
    /// - `/drone/sensors/accel/x`  
    /// - `/drone/sensors/accel/y`  
    /// - `/drone/sensors/accel/y/raw` Error: sub-signal of `/drone/sensors/accel/y`  
    /// - `drone/sensors/press1` Error: Does not start with `/`  
    /// - `/drone/sensors/temp-bat` Error: illegal character `-`  
    /// - `/drone/sensors//current` Error: consecutive `/`  
    pub fn add_signal(&mut self, name: &str) -> Result<(SignalID, SignalProducer), SignalError> {
        self.validate_name(name)?;

        let id = SignalID {
            id: Self::get_name_hash(name),
        };

        let (sender, receiver) = channel();

        if self.signals.contains_key(&id) {
            panic!("Signal ID hash collision");
        }

        self.signals.insert(id, Signal::new(name.to_string(), id));
        self.receivers.insert(id, receiver);

        Ok((id, SignalProducer { sender, id }))
    }

    pub fn update(&mut self) {
        for (id, signal) in self.signals.iter_mut() {
            let receiver = self.receivers.get(id).unwrap();

            while let Ok(sample) = receiver.try_recv() {
                signal.time.push(sample.time);
                signal.data.push(sample.value);
            }
        }
    }

    fn get_name_hash(name: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(name.as_bytes());
        hasher.finish()
    }
}

impl Signals {
    fn validate_name(&self, name: &str) -> Result<(), SignalError> {
        if !name.starts_with('/') {
            return Err(SignalError::NameError {
                name: name.to_string(),
                msg: "Signal name must start with a `/`.".to_string(),
            });
        }

        if name.contains("//") {
            return Err(SignalError::NameError {
                name: name.to_string(),
                msg: "Signal name must not contain two or more consecutive `/`.".to_string(),
            });
        }

        let regex = Regex::new(r"^[\w\/]+$").unwrap();

        if !regex.is_match(name) {
            return Err(SignalError::NameError {
                name: name.to_string(),
                msg: "Signal name must contain only letters, numbers, underscore or `/`.".to_string(),
            });
        }

        // Check that the signal name is not a subpath of an existing signal
        for sig_name in self.signals.values().map(|sig| &sig.name) {
            if name.starts_with(sig_name) {
                return Err(SignalError::NameError {
                    name: name.to_string(),
                    msg: format!(
                        "Provided signal name is a subsignal of {sig_name}"
                    )
                    .to_string(),
                });
            }
        }

        Ok(())
    } 
}

#[derive(Debug, Error)]
pub enum SignalError {
    #[error("Bad signal name: {msg}. Signal: '{name}'")]
    NameError { name: String, msg: String },
}

#[derive(Debug, Error)]
pub struct SignalSendError<T> {
    pub t: T,
}

impl<T> From<SendError<T>> for SignalSendError<T> {
    fn from(value: SendError<T>) -> Self {
        Self { t: value.0 }
    }
}

#[derive(Clone, Debug)]
pub struct SignalProducer {
    sender: Sender<SignalSample>,
    id: SignalID,
}

impl SignalProducer {
    pub fn send(&mut self, sample: SignalSample) -> Result<(), SignalSendError<SignalSample>> {
        self.sender.send(sample).map_err(|e| e.into())
    }

    pub fn id(&self) -> SignalID {
        self.id
    }
}
