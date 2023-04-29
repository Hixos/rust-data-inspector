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
    pub receiver: Receiver<SignalSample>,
}

pub struct SignalGroup {
    signals: HashMap<String, SignalHandle>,
    receiver: Receiver<SignalHandle>,
    name_tree: SimpleTree<NameNode>
}

impl SignalGroup {
    pub fn new(receiver: Receiver<SignalHandle>) -> Self {
        SignalGroup {
            signals: HashMap::new(),
            receiver,
            name_tree: SimpleTree::new(NameNode {
                name: "".into(),
                path: "".into(),
                is_signal: false,
            })
        }
    }

    pub fn update(&mut self) {
        for handle in self.receiver.try_iter() {
            let name = handle.signal.name().to_string();
            self.signals.insert(name.clone(), handle);

            if Self::tree_insert(&mut self.name_tree, name).is_err() {
                panic!("Insertion in tree failed!");
            }
        }

        for handle in self.signals.values_mut() {
            for sample in handle.receiver.try_iter() {
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
                Some(ts) => {
                    match &max {
                        Some(max_val) => {
                            if ts > max_val {
                                max = Some(*ts);
                            }
                        }
                        None => {
                            max = Some(*ts);
                        }
                    }
                }
                None => {}
            }
        }

        max
    }

    fn tree_insert(tree: &mut SimpleTree<NameNode>, name: String) -> Result<(), ()> {
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
                            return Self::tree_insert(n, second.into());
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

                Self::tree_insert(child, second.into())
            }
            None => {
                // We are a leaf, check if there are no other leafs with the same name, otherwise add it
                if tree.get_children_mut().iter().any(|n| n.elem.name == name) {
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
}
