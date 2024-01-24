use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Receiver, SendError, Sender};

use regex::Regex;
use serde::{Serialize, Deserialize};
use thiserror::Error;

pub struct PlotSignal {
    id: PlotSignalID,
    name: String,

    time: Vec<f64>,
    data: Vec<f64>,
}

impl PlotSignal {
    pub fn new(name: String, id: PlotSignalID) -> Self {
        PlotSignal {
            id,
            name,
            time: vec![],
            data: vec![],
        }
    }

    pub fn id(&self) -> PlotSignalID {
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct PlotSignalID {
    id: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct PlotSignalSample {
    pub time: f64,
    pub value: f64,
}

impl Display for PlotSignalSample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.time, self.value)
    }
}

#[derive(Default)]
pub struct PlotSignals {
    signals: HashMap<PlotSignalID, PlotSignal>,
    receivers: HashMap<PlotSignalID, Receiver<PlotSignalSample>>,
}

impl PlotSignals {
    pub fn get_signals(&self) -> &HashMap<PlotSignalID, PlotSignal> {
        &self.signals
    }

    /// Returns the signal with the provided id.  
    /// Assumes that the SignalID was provided a call to add_signal of this instance.  
    /// Panics if a signal with the provided id is not found. This happens if the id belongs to another instance.
    pub fn get_signal(&self, id: PlotSignalID) -> &PlotSignal {
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
    pub fn add_signal(&mut self, name: &str) -> Result<(PlotSignalID, PlotSampleSender), PlotSignalError> {
        self.validate_name(name)?;

        let id = PlotSignalID {
            id: Self::get_name_hash(name),
        };

        let (sender, receiver) = channel();

        if self.signals.contains_key(&id) {
            panic!("Signal ID hash collision");
        }

        self.signals.insert(id, PlotSignal::new(name.to_string(), id));
        self.receivers.insert(id, receiver);

        Ok((id, PlotSampleSender { sender, id }))
    }

    pub fn update(&mut self) {
        for (id, signal) in self.signals.iter_mut() {
            let receiver = self.receivers.get(id).unwrap();

            while let Ok(sample) = receiver.try_recv() {
                if let Some(&last) = signal.time.last()  {
                    if sample.time < last {
                        panic!("Received sample in the past!");
                    }
                }
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

impl PlotSignals {
    fn validate_name(&self, name: &str) -> Result<(), PlotSignalError> {
        if !name.starts_with('/') {
            return Err(PlotSignalError::NameError {
                name: name.to_string(),
                msg: "Signal name must start with a `/`.".to_string(),
            });
        }

        if name.contains("//") {
            return Err(PlotSignalError::NameError {
                name: name.to_string(),
                msg: "Signal name must not contain two or more consecutive `/`.".to_string(),
            });
        }

        let regex = Regex::new(r"^[\w\/]+$").unwrap();

        if !regex.is_match(name) {
            return Err(PlotSignalError::NameError {
                name: name.to_string(),
                msg: "Signal name must contain only letters, numbers, underscore or `/`.".to_string(),
            });
        }

        // Check that the signal name is not a subpath of an existing signal
        for sig_name in self.signals.values().map(|sig| &sig.name) {
            if name.starts_with(sig_name) {
                return Err(PlotSignalError::NameError {
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
pub enum PlotSignalError {
    #[error("Bad signal name: {msg}. Signal: '{name}'")]
    NameError { name: String, msg: String },
}

#[derive(Debug, Error)]
#[error("Error sending plot signal sample '{t}'")]
pub struct PlotSignalSendError<T> {
    pub t: T,
}

impl<T> From<SendError<T>> for PlotSignalSendError<T> {
    fn from(value: SendError<T>) -> Self {
        Self { t: value.0 }
    }
}

#[derive(Clone, Debug)]
pub struct PlotSampleSender {
    sender: Sender<PlotSignalSample>,
    id: PlotSignalID,
}

impl PlotSampleSender {
    pub fn send(&mut self, sample: PlotSignalSample) -> Result<(), PlotSignalSendError<PlotSignalSample>> {
        self.sender.send(sample).map_err(|e| e.into())
    }

    pub fn id(&self) -> PlotSignalID {
        self.id
    }
}
