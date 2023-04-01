use super::signal::{Signal, SignalSample};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
// use super::util::SimpleTree;

pub struct SignalHandle {
    pub signal: Signal,
    pub receiver: Receiver<SignalSample>,
}

pub struct SignalGroup {
    signals: HashMap<String, SignalHandle>,
    receiver: Receiver<SignalHandle>,
    // name_tree: SimpleTree<String>
}

impl SignalGroup {
    pub fn new(receiver: Receiver<SignalHandle>) -> Self {
        SignalGroup {
            signals: HashMap::new(),
            receiver,
            // name_tree: SimpleTree::new("/".into())
        }
    }

    pub fn update(&mut self) {
        for handle in self.receiver.try_iter() {
            self.signals
                .insert(handle.signal.name().to_string(), handle);
        }

        for handle in self.signals.values_mut() {
            for sample in handle.receiver.try_iter() {
                handle.signal.push(sample);
            }
        }
    }

    pub fn get_signal(&self, key: &str) -> Option<&Signal> {
        self.signals.get(key).and_then(|h| Some(&h.signal))
    }
}
