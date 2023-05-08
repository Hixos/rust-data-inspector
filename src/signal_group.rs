use egui::Color32;
use egui::epaint::Hsva;
use serde::{Deserialize, Serialize};

use super::signal::{Signal, SignalSample};
use super::util::SimpleTree;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct NameNode {
    pub name: String,
    pub path: String,
    pub is_signal: bool,
}

pub struct SignalHandle {
    pub signal: Signal,
    pub on_new_sample: Receiver<SignalSample>,
}

pub struct SignalGroup {
    signals: HashMap<String, SignalHandle>,
    on_new_signal: Receiver<SignalHandle>,
    name_tree: SimpleTree<NameNode>,
    pub signal_data: SignalData,
    next_auto_color_idx: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SignalData {
    pub colors: HashMap<String, Color32>,
}

impl Default for SignalData {
    fn default() -> Self {
        SignalData {
            colors: HashMap::new(),
        }
    }
}

impl SignalGroup {
    pub fn new(receiver: Receiver<SignalHandle>) -> Self {
        SignalGroup {
            signals: HashMap::new(),
            on_new_signal: receiver,
            name_tree: SimpleTree::new(NameNode {
                name: "".into(),
                path: "".into(),
                is_signal: false,
            }),
            signal_data: SignalData::default(),
            next_auto_color_idx: 0,
        }
    }

    pub fn update(&mut self) {
        for handle in self.on_new_signal.try_iter() {
            let name = handle.signal.name().to_string();
            self.signals.insert(name.clone(), handle);

            if Self::tree_insert(&mut self.name_tree, &name).is_err() {
                panic!("Insertion in tree failed!");
            }

            if !self.signal_data.colors.contains_key(&name) {
                self.signal_data.colors.insert(name, Self::auto_color(self.next_auto_color_idx));
                self.next_auto_color_idx += 1;
            }
        }

        for handle in self.signals.values_mut() {
            for sample in handle.on_new_sample.try_iter() {
                handle.signal.push(sample);
            }
        }
    }

    pub fn get_signal<S: Into<String>>(&self, key: S) -> Option<&Signal> {
        self.signals.get(&key.into()).and_then(|h| Some(&h.signal))
    }

    pub fn get_tree(&self) -> &SimpleTree<NameNode> {
        &self.name_tree
    }

    pub fn current_timestamp(&self) -> Option<f64> {
        let mut max: Option<f64> = None;

        for (_, signal) in &self.signals {
            let ts = signal.signal.time().last();
            match ts {
                Some(ts) => match &max {
                    Some(max_val) => {
                        if ts > max_val {
                            max = Some(*ts);
                        }
                    }
                    None => {
                        max = Some(*ts);
                    }
                },
                None => {}
            }
        }

        max
    }

    pub fn initial_timestamp(&self) -> Option<f64> {
        let mut min: Option<f64> = None;

        for (_, signal) in &self.signals {
            let ts = signal.signal.time().first();
            match ts {
                Some(ts) => match &min {
                    Some(min_val) => {
                        if ts < min_val {
                            min = Some(*ts);
                        }
                    }
                    None => {
                        min = Some(*ts);
                    }
                },
                None => {}
            }
        }

        min
    }

    fn tree_insert(tree: &mut SimpleTree<NameNode>, name: &String) -> Result<(), ()> {
        let join_path = |a: &str, b: &str| {
            if a != "" {
                return [a.to_string(), b.to_string()].join("/");
            }
            b.into()
        };

        match name.split_once("/") {
            Some((first, second)) => {
                for n in tree.get_children_mut().iter_mut() {
                    let elem = &n.elem;
                    if elem.name == first {
                        if !elem.is_signal {
                            return Self::tree_insert(n, &second.to_string());
                        } else {
                            // Signal nodes cannot have additional children
                            return Err(());
                        }
                    }
                }

                // No node found. Add it
                let path = join_path(tree.elem.path.as_str(), first);

                let child = tree.add_child(NameNode {
                    name: first.into(),
                    path: path,
                    is_signal: false,
                });

                Self::tree_insert(child, &second.to_string())
            }
            None => {
                // We are a leaf, check if there are no other leafs with the same name, otherwise add it
                if tree.get_children_mut().iter().any(|n| n.elem.name == *name) {
                    return Err(());
                } else {
                    tree.add_child(NameNode {
                        name: name.clone(),
                        path: join_path(tree.elem.path.as_str(), name.as_str()),
                        is_signal: true,
                    });
                    return Ok(());
                }
            }
        }
    }
    
    fn auto_color(color_idx: usize) -> Color32 {
        let i = color_idx;
        let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
        let h = i as f32 * golden_ratio;
        Hsva::new(h, 0.85, 0.5, 1.0).into()
    }
}
