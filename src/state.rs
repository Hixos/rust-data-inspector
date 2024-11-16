use std::{
    collections::{BTreeSet, HashMap},
    ops::RangeInclusive,
};

use crate::layout::tabviewer::BaseTab;
use crate::utils::downsampling::DownsamplingMethod;
use eframe::Storage;
use egui::Color32;
use egui_dock::DockState;
use rust_data_inspector_signals::{PlotSignalID, PlotSignals};
use serde::{Deserialize, Serialize};

use crate::{
    layout::tabviewer::Tab,
    utils::{auto_color, VecTree},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataInspectorState {
    // Plots
    pub visible_range: RangeInclusive<f64>,

    // Timeseries
    pub downsample_mode: DownsamplingMethod,
    pub x_axis_mode: XAxisMode,
    pub link_x: bool,

    // XY
    pub xy_num_samples: usize,

    // Dock
    pub selected_pane: u64,

    // Signals
    pub signal_state: HashMap<PlotSignalID, SignalState>,
    pub signal_color_counter: usize,

    #[serde(skip)]
    pub debug_info: DebugInfo,
}

impl DataInspectorState {
    pub fn new(signals: &PlotSignals) -> Self {
        DataInspectorState {
            visible_range: RangeInclusive::new(0.0, 0.0),
            xy_num_samples: 1000,
            x_axis_mode: XAxisMode::default(),
            link_x: true,
            downsample_mode: DownsamplingMethod::Lttb,
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
            signal_color_counter: signals.get_signals().len(),
            debug_info: DebugInfo::default(),
        }
    }

    pub fn from_storage(storage: &dyn Storage, signals: &PlotSignals) -> Option<Self> {
        if let Some(mut slf) = eframe::get_value::<Self>(storage, "state") {
            // Remove state of signals that are not present anymore
            slf.signal_state
                .retain(|id, _| signals.get_signals().contains_key(id));

            signals
                .get_signals()
                .iter()
                .enumerate()
                .for_each(|(i, (id, _))| {
                    if !slf.signal_state.contains_key(id) {
                        slf.signal_state.insert(
                            *id,
                            SignalState {
                                color: auto_color(i + slf.signal_color_counter),
                                used_by_tile: BTreeSet::new(),
                            },
                        );
                        slf.signal_color_counter += 1;
                    }
                });

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
    pub tree: DockState<BaseTab>,
    pub tab_counter: u64,
}

impl Default for TabState {
    fn default() -> Self {
        let tree = DockState::new(vec![BaseTab::new(1, Tab::timeseries(1))]);
        TabState {
            tree,
            tab_counter: 2,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DebugInfo {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalState {
    pub color: Color32,
    pub used_by_tile: BTreeSet<u64>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
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
    pub signal: Option<PlotSignalID>,
}

pub struct SignalData {
    signals: PlotSignals,
    signal_tree: VecTree<SignalNode>,

    time_span: Option<[f64; 2]>,
    pub all_signals_have_data: bool,
}

impl SignalData {
    pub fn new(signals: PlotSignals) -> Self {
        let signal_tree = Self::grow_signal_tree(&signals);

        SignalData {
            signals,
            signal_tree,
            time_span: None,
            all_signals_have_data: false,
        }
    }

    pub fn signals(&self) -> &PlotSignals {
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
            if !self.all_signals_have_data {
                if let Some(&first) = sig.time().first() {
                    if min.is_none() || first < min.unwrap() {
                        min = Some(first);
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

        self.all_signals_have_data = self
            .signals
            .get_signals()
            .iter()
            .all(|(_, s)| !s.time().is_empty());
    }

    fn grow_signal_tree(signals: &PlotSignals) -> VecTree<SignalNode> {
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
