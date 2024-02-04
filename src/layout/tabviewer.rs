use egui_dock::{NodeIndex, SurfaceIndex};
use serde::{Deserialize, Serialize};

use crate::state::{DataInspectorState, SignalData};

use super::tabs::timeseries::TimeSeriesTab;

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseTab {
    pub tab_id: u64,
    pub tab: Tab,
}

impl BaseTab {
    pub fn new(tab_id: u64, tab: Tab) -> Self {
        Self { tab_id, tab }
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut DataInspectorState,
        signals: &mut SignalData,
        link_x_translated: &mut bool,
    ) {
        match &mut self.tab {
            Tab::TimeSeries(ts) => {
                ts.ui(ui, state, signals, link_x_translated);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Tab {
    TimeSeries(TimeSeriesTab),
}

impl Tab {
    pub fn timeseries(tab_id: u64) -> Self {
        Tab::TimeSeries(TimeSeriesTab::new(tab_id))
    }
}

pub struct TabViewer<'a> {
    state: &'a mut DataInspectorState,
    signals: &'a mut SignalData,

    link_x_translated: bool,

    pub added_nodes: Vec<(SurfaceIndex, NodeIndex, Box<dyn FnOnce(u64) -> Tab>)>,
}

impl<'a> TabViewer<'a> {
    pub fn new(state: &'a mut DataInspectorState, signals: &'a mut SignalData) -> Self {
        TabViewer {
            state,
            signals,
            link_x_translated: false,
            added_nodes: vec![],
        }
    }
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = BaseTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("Tab {}", tab.tab_id).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(ui, self.state, self.signals, &mut self.link_x_translated);
    }

    // fn on_add(&mut self, surface: egui_dock::SurfaceIndex, node: egui_dock::NodeIndex) {
    //     self.added_nodes.push((surface, node));
    // }

    fn add_popup(&mut self, ui: &mut egui::Ui, surface: SurfaceIndex, node: NodeIndex) {
        ui.set_min_width(80.0);
        ui.style_mut().visuals.button_frame = false;

        if ui.button("Timeseries").clicked() {
            self.added_nodes
                .push((surface, node, Box::new(Tab::timeseries)));
        }
    }
}
