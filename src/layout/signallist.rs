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
        Self::ui_impl(ui, signals.signal_tree(), state, true);
    }

    fn ui_impl(
        ui: &mut egui::Ui,
        node: &VecTree<SignalNode>,
        state: &mut DataInspectorState,
        is_root: bool,
    ) {
        if node.children.is_empty() {
            let id = node.value.signal.unwrap();
            let signal_state = state.signal_state.get_mut(&id).unwrap();

            let col = &mut signal_state.color;
            let mut srgb = [col.r(), col.g(), col.b()];

            let selected = signal_state.used_by_tile.contains(&state.selected_pane);
            let mut selected_mut = selected;
            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut srgb);
                ui.toggle_value(&mut selected_mut, node.value.name.clone());
            });

            *col = Color32::from_rgb(srgb[0], srgb[1], srgb[2]);
            if selected_mut != selected {
                // Value was changed
                if selected_mut {
                    signal_state.used_by_tile.insert(state.selected_pane);
                } else {
                    signal_state.used_by_tile.remove(&state.selected_pane);
                }
            }
        } else if is_root {
            for child in node.children.iter() {
                Self::ui_impl(ui, child, state, false);
            }
        } else {
            CollapsingHeader::new(node.value.name.clone())
                .id_source(node.value.path.clone())
                .default_open(true)
                .show(ui, |ui| {
                    for child in node.children.iter() {
                        Self::ui_impl(ui, child, state, false);
                    }
                });
        }
    }
}
