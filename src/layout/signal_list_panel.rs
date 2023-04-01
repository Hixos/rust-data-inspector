use crate::signal_group::SignalGroup;

pub struct SignalList {}

impl SignalList {
    pub fn new() -> Self {
        SignalList {}
    }

    pub fn ui(&self, ui: &mut egui::Ui, signals: &SignalGroup) {
        ui.heading("Signals");
        
    }

    fn collapsing_tree(&self, ui: *mut egui::Ui) {

    }
}
