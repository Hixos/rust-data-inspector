use std::collections::{BTreeSet, HashMap};

use egui::Color32;
use rust_data_inspector_signals::{SignalID, Signals};

use crate::utils::VecTree;

#[derive(Debug)]
pub struct DataInspectorState {
    pub x_axis_mode: XAxisMode,
    pub link_x: bool,

    pub selected_tile: u64,
    pub tile_counter: u64,
    pub signal_state: HashMap<SignalID, SignalState>,
    pub pane_state: HashMap<u64, TileState>,
}

impl DataInspectorState {
    pub fn new(signals: &Signals) -> Self {
        DataInspectorState {
            x_axis_mode: XAxisMode::default(),
            link_x: true,
            selected_tile: 0,
            tile_counter: 0,
            signal_state: signals
                .get_signals()
                .iter()
                .map(|(id, _)| {
                    (
                        *id,
                        SignalState {
                            color: Color32::BLUE,
                            used_by_tile: BTreeSet::new(),
                        },
                    )
                })
                .collect(),
            pane_state: HashMap::new(),
        }
    }

    pub fn get_pane_id_and_increment(&mut self) -> u64 {
        let id = self.tile_counter;
        self.tile_counter += 1;
        id
    }
}

#[derive(Debug, Default)]
pub struct TileState {
    pub plot_transformed: bool,
}

#[derive(Debug)]
pub struct SignalState {
    pub color: Color32,
    pub used_by_tile: BTreeSet<u64>,
}

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum XAxisMode {
    #[default]
    Fit,
    Follow,
    Free,
}

#[derive(Debug)]
pub struct SignalNode {
    pub name: String,
    pub path: String,
    pub signal: Option<SignalID>,
}

pub struct SignalData {
    pub signals: Signals,
    pub signal_tree: VecTree<SignalNode>,
}

impl SignalData {
    pub fn new(signals: Signals) -> Self {
        let signal_tree = Self::grow_signal_tree(&signals);

        SignalData {
            signals,
            signal_tree,
        }
    }

    fn grow_signal_tree(signals: &Signals) -> VecTree<SignalNode> {
        fn node_contains(node: &VecTree<SignalNode>, part: &str) -> bool {
            for n in node.nodes_iter() {
                if n.v.name == part {
                    return true;
                }
            }

            false
        }

        let mut root = VecTree::new(SignalNode {
            name: "".to_string(),
            path: "/".to_string(),
            signal: None,
        });

        for (id, signal) in signals.get_signals().iter() {
            let mut node = &mut root;

            let mut path = "/".to_string();
            let parts: Vec<_> = signal.name().split('/').skip(1).collect();

            for part in parts.iter().take(parts.len() - 1).map(|v| v.to_string()) {
                path.push_str(&part);
                path.push('/');

                if !node_contains(node, part.as_str()) {
                    node.push(SignalNode {
                        name: part.clone(),
                        path: path.clone(),
                        signal: None,
                    });
                }
                node = node.nodes_iter_mut().last().unwrap();
            }
            // Assumption: Signal names are legal and a signal does not have any subnodes
            let last = parts.last().unwrap();
            path.push_str(last);
            node.push(SignalNode {
                name: last.to_string(),
                path,
                signal: Some(*id),
            });
        }

        root
    }
}
