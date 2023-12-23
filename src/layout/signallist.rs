use crate::utils::SimpleTree;

struct SignalList {}

impl SignalList {
    pub fn new() -> SignalList {
        SignalList {}
    }

    pub fn ui(&self, ui: &mut egui::Ui, tree: &SimpleTree<String>) {
        self.ui_impl(ui, tree);
    }

    fn ui_impl(&self, ui: &mut egui::Ui, node: &SimpleTree<String>) {
        if node.get_children().is_empty() {
            ui.label(node.elem.clone());
        } else {
            for child in node.get_children().iter() {
                ui.collapsing(node.elem.clone(), |ui| self.ui_impl(ui, child));
            }
        }
    }
}
