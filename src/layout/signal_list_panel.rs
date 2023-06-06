use std::collections::{HashSet, HashMap};

use egui::Color32;

use crate::{
    signal_group::{NameNode, SignalGroup},
    util::SimpleTree,
};

pub struct SignalList {}

impl SignalList {
    pub fn new() -> Self {
        SignalList {}
    }

    pub fn ui(&self, ui: &mut egui::Ui, signals: &mut SignalGroup, enabled_signals: &mut HashSet<String>) {
        ui.heading("Signals");

        let mut colors =  signals.signal_data.colors.clone();
        self.signal_tree_ui(ui, signals.get_tree(), &mut colors, enabled_signals);

        for (key, col) in colors.iter() {
            signals.signal_data.colors.insert(key.clone(), col.clone());
        }
    }

    fn signal_tree_ui(&self, ui: &mut egui::Ui, tree: &SimpleTree<NameNode>, colors: &mut HashMap<String, Color32>, enabled_signals: &mut HashSet<String>) {
        for child in tree.get_children() {
            let e = &child.elem;
            let mut selected = enabled_signals.contains(&e.path);

            if e.is_signal {
                let col = colors.get_mut(&e.path).unwrap(); // Ok to panic if signal has no associated color
                let mut col_srgb = [col.r(), col.g(), col.b()];
                
                ui.horizontal(|ui| {
                    ui.color_edit_button_srgb(&mut col_srgb);
                    ui.toggle_value(&mut selected, e.name.clone());
                });

                *col = Color32::from_rgb(col_srgb[0], col_srgb[1], col_srgb[2]);
            } else {
                ui.collapsing(e.name.clone(), |ui| self.signal_tree_ui(ui, child, colors, enabled_signals));
            }

            if selected {
                enabled_signals.insert(e.path.clone());
            }else{
                enabled_signals.remove(&e.path);
            }
        }
    }
}
