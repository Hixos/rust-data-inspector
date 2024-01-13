use egui::{vec2, Event, Sense, Stroke, TextStyle, Vec2, Vec2b};
use egui_plot::{Line, PlotBounds, PlotPoint, PlotPoints, VPlacement};
use egui_tiles::{SimplificationOptions, Tile, TileId};

use crate::state::{DataInspectorState, SignalData, XAxisMode};

const DEFAULT_PLOT_WIDTH: f64 = 30.0;
const PLOT_MARGIN_PC: f64 = 0.01;

#[derive(Debug, Default)]
pub struct Pane {
    pub id: u64,
}

impl Pane {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut DataInspectorState, signals: &mut SignalData) {
        let (scroll, pointer_down, modifiers) = ui.input(|i| {
            let scroll = i.events.iter().find_map(|e| match e {
                Event::MouseWheel {
                    unit: _,
                    delta,
                    modifiers: _,
                } => Some(*delta),
                _ => None,
            });
            (scroll, i.pointer.primary_down(), i.modifiers)
        });

        let mut plot = egui_plot::Plot::new(format!("plot_{}", self.id));
        let response = plot
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_double_click_reset(false)
            .auto_bounds(Vec2b::FALSE)
            .show(ui, |plot_ui| {
                let mut left_most: Option<f64> = None;
                let mut right_most: Option<f64> = None;

                for (id, signal) in signals.signals.get_signals() {
                    if state
                        .signal_state
                        .get(id)
                        .unwrap()
                        .used_by_tile
                        .contains(&self.id)
                    {
                        if let Some(last) = signal.time().last() {
                            if right_most.is_none() || right_most.unwrap() < *last {
                                right_most = Some(*last);
                            }
                        }

                        if let Some(first) = signal.time().first() {
                            if left_most.is_none() || left_most.unwrap() > *first {
                                left_most = Some(*first);
                            }
                        }

                        let points = signal
                            .time()
                            .iter()
                            .zip(signal.data().iter())
                            .map(|(t, v)| [*t, *v])
                            .collect::<PlotPoints>();

                        plot_ui.line(Line::new(points));
                    }
                }

                let mut transformed = false;

                // Plot mode transformations
                match state.x_axis_mode {
                    XAxisMode::Follow => {
                        let bounds = plot_ui.plot_bounds();
                        let dx = right_most.unwrap_or(DEFAULT_PLOT_WIDTH) - bounds.max()[0]
                            + bounds.width() * PLOT_MARGIN_PC;
                        plot_ui.translate_bounds(Vec2 {
                            x: dx as f32,
                            y: 0.0,
                        });
                    }

                    XAxisMode::Fit => {
                        let bounds = plot_ui.plot_bounds();
                        let bounds = PlotBounds::from_min_max(
                            [
                                left_most.unwrap_or(0.0) - bounds.width() * PLOT_MARGIN_PC,
                                bounds.min()[1],
                            ],
                            [
                                right_most.unwrap_or(DEFAULT_PLOT_WIDTH)
                                    + bounds.width() * PLOT_MARGIN_PC,
                                bounds.max()[1],
                            ],
                        );

                        plot_ui.set_plot_bounds(bounds);
                    }

                    XAxisMode::Free => {}
                }

                // User interaction transformations
                if plot_ui.response().hovered() {
                    if let Some(mut scroll) = scroll {
                        scroll = Vec2::splat(scroll.x + scroll.y);
                        let mut zoom_factor =
                            Vec2::from([(scroll.x / 10.0).exp(), (scroll.y / 10.0).exp()]);

                        if modifiers.ctrl {
                            zoom_factor.y = 1.0;
                        } else {
                            zoom_factor.x = 1.0;
                        }

                        match state.x_axis_mode {
                            XAxisMode::Free => plot_ui.zoom_bounds_around_hovered(zoom_factor),
                            XAxisMode::Follow => {
                                if let Some(mut pointer_coord) = plot_ui.pointer_coordinate() {
                                    pointer_coord.x = plot_ui.plot_bounds().max()[0];
                                    plot_ui.zoom_bounds(zoom_factor, pointer_coord);
                                }
                            }
                            XAxisMode::Fit => {
                                zoom_factor.x = 1.0;
                                plot_ui.zoom_bounds_around_hovered(zoom_factor);
                            }
                        }

                        transformed = true;
                    }

                    if pointer_down {
                        let mut pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                        if state.x_axis_mode != XAxisMode::Free {
                            pointer_translate.x = 0.0;
                        }
                        // if self.lock_y {
                        //     pointer_translate.y = 0.0;
                        // }
                        plot_ui.translate_bounds(pointer_translate);
                    }
                }

                state.pane_state.get_mut(&self.id).unwrap().plot_transformed = transformed;
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
