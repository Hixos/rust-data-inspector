use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Signal {
    id: SignalID,
    name: String,

    pub time: Vec<f64>,
    pub data: Vec<f64>,
}

impl Signal {
    pub fn new(name: &str) -> Self {
        let mut name = name.to_owned();

        if !name.starts_with('/') {
            name = "/".to_owned() + &name;
        }

        
    }

    pub fn id(&self) -> SignalID {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn path_elements(&self) -> Vec<&str> {
        self.name.split('/').collect::<Vec<&str>>()
    }
}


#[derive(Clone, Copy)]
pub struct SignalSample {
    pub time: f64,
    pub value: f64,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct SignalID {
    id: u64,
}

impl SignalID {
    fn from_name(name: &str) -> SignalID {
        let mut h = DefaultHasher::new();
        name.hash(&mut h);
        SignalID { id: h.finish() }
    }
}


#[derive(Default)]
pub struct SignalGroup {
    signals: HashMap<SignalID, Signal>,
    signals_data: HashMap<SignalID, SignalData>,
}

struct SignalData {
    sample_receiver: Receiver<SignalSample>,
}

impl SignalGroup {
    pub fn get_signals(&self) -> &HashMap<SignalID, Signal> {
        &self.signals
    }

    pub fn get_signal(&self, id: SignalID) -> Option<&Signal> {
        self.signals.get(&id)
    }

    pub fn add_signal(&mut self, name: &str) -> Sender<SignalSample> {

        let id = ;

        let (sender, receiver) = channel();

        self.signals.insert(
            id,
            Signal {
                id,
                name: name.to_owned(),
                time: vec![],
                data: vec![],
            },
        );

        self.signals_data.insert(
            id,
            SignalData {
                sample_receiver: receiver,
            },
        );

        sender
    }

    pub fn update_signals(&mut self) {
        for (id, signal) in self.signals.iter_mut() {
            let receiver = &self.signals_data.get(id).unwrap().sample_receiver;

            while let Ok(sample) = receiver.try_recv() {
                signal.time.push(sample.time);
                signal.data.push(sample.value);
            }
        }
    }
}
