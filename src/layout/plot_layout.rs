use crate::{widget::RTPlot, SignalGroup};
use std::collections::HashSet;

pub struct PlotTab {
    pub signals: HashSet<String>,

    index: usize,
}

impl PlotTab {
    pub fn new(index: usize) -> Self {
        PlotTab {
            index,
            signals: HashSet::new(),
        }
    }

    pub fn title(&self) -> String {
        ["Tab".to_string(), self.index.to_string()].join(" ")
    }
}

pub struct PlotLayout {
    pub tree: egui_dock::Tree<PlotTab>,

    tab_counter: usize,
}

impl PlotLayout {
    pub fn new() -> Self {
        let tree = egui_dock::Tree::new(vec![PlotTab::new(1)]);

        PlotLayout {
            tree,
            tab_counter: 2usize,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, signals: &SignalGroup) {
        let mut style = egui_dock::Style::from_egui(ui.style().as_ref());
        style.show_add_buttons = true;

        let mut tab_viewer = PlotTabViewer::new(signals);

        egui_dock::DockArea::new(&mut self.tree)
            .style(style)
            .show_inside(ui, &mut tab_viewer);

        match tab_viewer.add_tab {
            Some(index) => {
                let test = &mut self.tree[index];
                test.append_tab(PlotTab::new(self.tab_counter));
                self.tab_counter += 1;
            }
            None => {}
        }
        

    }
}

struct PlotTabViewer<'a> {
    add_tab: Option<egui_dock::NodeIndex>,
    signals: &'a SignalGroup
}

impl<'a> PlotTabViewer<'a> {
    pub fn new(signals: &'a SignalGroup) -> Self {
        PlotTabViewer { add_tab: None, signals }
    }
}

impl<'a> egui_dock::TabViewer for PlotTabViewer<'a> {
    type Tab = PlotTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let mut plot = RTPlot::new(tab.index.to_string());

        let sig = tab.signals.iter().next();
        if sig.is_some() {
            plot.show(ui, | plot_ui | {
                
            });
        }
    }

    fn on_add(&mut self, node: egui_dock::NodeIndex) {
        self.add_tab = Some(node);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab.title()).into()
    }
}

