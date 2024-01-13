use std::collections::{BTreeSet, HashMap};

use eframe::Storage;
use egui::Color32;
use egui_dock::DockState;
use rust_data_inspector_signals::{SignalID, Signals};
use serde::{Deserialize, Serialize};

use crate::{
    layout::tabs::Tab,
    utils::{auto_color, VecTree},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataInspectorState {
    pub x_axis_mode: XAxisMode,
    pub link_x: bool,

    pub selected_pane: u64,
    pub signal_state: HashMap<SignalID, SignalState>,
}

impl DataInspectorState {
    pub fn new(signals: &Signals) -> Self {
        DataInspectorState {
            x_axis_mode: XAxisMode::default(),
            link_x: true,
            selected_pane: 1,
            signal_state: signals
                .get_signals()
                .iter()
                .enumerate()
                .map(|(i, (id, _))| {
                    (
                        *id,
                        SignalState {
                            color: auto_color(i),
                            used_by_tile: BTreeSet::new(),
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn from_storage(storage: &dyn Storage, signals: &Signals) -> Option<Self> {
        if let Some(mut slf) = eframe::get_value::<Self>(storage, "state") {
            // Remove state of signals that are not present anymore
            slf.signal_state
                .retain(|id, _| signals.get_signals().contains_key(id));

            Some(slf)
        } else {
            None
        }
    }

    pub fn to_storage(&self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "state", self);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TabState {
    pub tree: DockState<Tab>,
    pub tab_counter: u64,
}

impl Default for TabState {
    fn default() -> Self {
        let tree = DockState::new(vec![Tab::new(1)]);
        TabState {
            tree,
            tab_counter: 2,
        }
    }
}

impl TabState {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalState {
    pub color: Color32,
    pub used_by_tile: BTreeSet<u64>,
}

#[derive(PartialEq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
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
    signals: Signals,
    signal_tree: VecTree<SignalNode>,

    time_span: Option<[f64; 2]>,
    all_signals_not_empty: bool,
}

impl SignalData {
    pub fn new(signals: Signals) -> Self {
        let signal_tree = Self::grow_signal_tree(&signals);

        SignalData {
            signals,
            signal_tree,
            time_span: None,
            all_signals_not_empty: false,
        }
    }

    pub fn signals(&self) -> &Signals {
        &self.signals
    }

    pub fn signal_tree(&self) -> &VecTree<SignalNode> {
        &self.signal_tree
    }

    pub fn time_span(&self) -> Option<[f64; 2]> {
        self.time_span
    }

    pub fn update(&mut self) {
        self.signals.update();

        for sig in self.signals.get_signals().values() {
            let mut min = self.time_span.map(|v| v[0]);
            let mut max = self.time_span.map(|v| v[1]);

            // Update initial time only until we have considered all signals
            if !self.all_signals_not_empty {
                self.all_signals_not_empty = true; // Tentatively set this to true
                if let Some(&first) = sig.time().first() {
                    if min.is_none() || first < min.unwrap() {
                        min = Some(first);
                    } else {
                        self.all_signals_not_empty = false;
                    }
                }
            }

            if let Some(&last) = sig.time().last() {
                if max.is_none() || last > max.unwrap() {
                    max = Some(last);
                }
            }

            if let (Some(min), Some(max)) = (min, max) {
                self.time_span = Some([min, max]);
            }
        }
    }

    fn grow_signal_tree(signals: &Signals) -> VecTree<SignalNode> {
        fn insert_ordered(
            node: &mut VecTree<SignalNode>,
            elem: SignalNode,
        ) -> Option<&mut VecTree<SignalNode>> {
            if let Err(index) = node
                .children
                .binary_search_by_key(&elem.name, |v| v.value.name.clone())
            {
                node.children.insert(index, VecTree::new(elem));
                node.children.get_mut(index)
            } else {
                None
            }
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

                if let Some(index) = node.children.iter().position(|n| n.value.name == part) {
                    node = node.children.get_mut(index).unwrap();
                } else {
                    node = insert_ordered(
                        node,
                        SignalNode {
                            name: part.clone(),
                            path: path.clone(),
                            signal: None,
                        },
                    )
                    .unwrap();
                }
            }

            let last = parts.last().unwrap();
            path.push_str(last);

            insert_ordered(
                node,
                SignalNode {
                    name: last.to_string(),
                    path,
                    signal: Some(*id),
                },
            )
            .expect("Duplicate signal in signal tree");
        }

        root
    }
}
