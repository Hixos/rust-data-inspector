use egui::{vec2, Sense, Stroke, TextStyle};
use egui_plot::{Line, PlotPoint, PlotPoints, VPlacement};
use egui_tiles::{SimplificationOptions, TileId, Tile};

use crate::state::{DataInspectorState, SignalData};

#[derive(Debug, Default)]
pub struct Pane {
    pub id: u64,
}

impl Pane {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut DataInspectorState, signals: &mut SignalData) {
        let mut plot = egui_plot::Plot::new(format!("plot_{}", self.id));
        let response = plot.show(ui, |ui| {
            for (id, signal) in signals.signals.get_signals() {
                if state
                    .signal_state
                    .get(id)
                    .unwrap()
                    .used_by_tile
                    .contains(&self.id)
                {
                    let points = signal
                        .time()
                        .iter()
                        .zip(signal.data().iter())
                        .map(|(t, v)| [*t, *v])
                        .collect::<PlotPoints>();

                    ui.line(Line::new(points));
                }
            }
        });
        if response.response.clicked()
            || response.response.dragged()
            || response.response.gained_focus()
        {
            state.selected_tile = self.id;
        }
    }
}

pub struct TilesBehavior<'a> {
    state: &'a mut DataInspectorState,
    signals: &'a mut SignalData,

    pub add_child_to: Option<TileId>,
    pub close_tab: Option<TileId>,
}

impl<'a> TilesBehavior<'a> {
    pub fn new(state: &'a mut DataInspectorState, signals: &'a mut SignalData) -> Self {
        TilesBehavior {
            state,
            signals,
            add_child_to: None,
            close_tab: None,
        }
    }
}

impl<'a> egui_tiles::Behavior<Pane> for TilesBehavior<'a> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("Pane {}", pane.id).into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        pane.ui(ui, self.state, self.signals);
        egui_tiles::UiResponse::None
    }

    fn on_tab_button(
        &mut self,
        tiles: &egui_tiles::Tiles<Pane>,
        tile_id: TileId,
        button_response: egui::Response,
    ) -> egui::Response {
        if button_response.triple_clicked() {
            self.close_tab = Some(tile_id);
        } else if button_response.clicked()
            || button_response.dragged()
            || button_response.gained_focus()
        {
            if let Some(Tile::Pane(pane)) = tiles.get(tile_id) {
                self.state.selected_tile = pane.id;
            }
        }
        button_response
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("+").clicked() {
            self.add_child_to = Some(tile_id);
        }
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}
