use egui::{CollapsingHeader, Color32};

use crate::{
    state::{DataInspectorState, SignalData, SignalNode},
    utils::VecTree,
};

pub struct SignalListUI {}

impl SignalListUI {
    pub fn new() -> SignalListUI {
        SignalListUI {}
    }

    pub fn ui(&self, ui: &mut egui::Ui, signals: &SignalData, state: &mut DataInspectorState) {
        Self::ui_impl(ui, signals.signal_tree(), state);
    }

    fn ui_impl(ui: &mut egui::Ui, node: &VecTree<SignalNode>, state: &mut DataInspectorState) {
        if node.count_children() == 0 {
            let id = node.v.signal.unwrap();
            let signal_state = state.signal_state.get_mut(&id).unwrap();

            let col = &mut signal_state.color;
            let mut srgb = [col.r(), col.g(), col.b()];

            let selected = signal_state.used_by_tile.contains(&state.selected_tile);
            let mut selected_mut = selected;
            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut srgb);
                ui.toggle_value(&mut selected_mut, node.v.name.clone());
            });

            *col = Color32::from_rgb(srgb[0], srgb[1], srgb[2]);
            if selected_mut != selected { // Value was changed
                if selected_mut {
                    signal_state.used_by_tile.insert(state.selected_tile);
                }else{
                    signal_state.used_by_tile.remove(&state.selected_tile);
                }
            }
        } else {
            CollapsingHeader::new(node.v.name.clone())
                .id_source(node.v.path.clone())
                .show(ui, |ui| {
                    for child in node.nodes_iter() {
                        Self::ui_impl(ui, child, state);
                    }
                });
        }
    }
}
