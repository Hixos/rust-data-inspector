use std::collections::HashSet;

use crate::{
    signal_group::{NameNode, SignalGroup},
    util::SimpleTree,
};

pub struct SignalList {}

impl SignalList {
    pub fn new() -> Self {
        SignalList {}
    }

    pub fn ui(&self, ui: &mut egui::Ui, signals: &SignalGroup, enabled_signals: &mut HashSet<String>) {
        ui.heading("Signals");
        self.signal_tree_ui(ui, signals.get_tree(), enabled_signals);
    }

    fn signal_tree_ui(&self, ui: &mut egui::Ui, tree: &SimpleTree<NameNode>, enabled_signals: &mut HashSet<String>) {
        for child in tree.get_children() {
            let e = &child.elem;
            let mut selected = enabled_signals.contains(&e.path);

            if e.is_signal {
                ui.toggle_value(&mut selected, e.name.clone());
            } else {
                ui.collapsing(e.name.clone(), |ui| self.signal_tree_ui(ui, child, enabled_signals));
            }

            if selected {
                enabled_signals.insert(e.path.clone());
            }else{
                enabled_signals.remove(&e.path);
            }
        }
    }
}
