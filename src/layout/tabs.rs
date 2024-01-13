use egui::{Event, Vec2, Vec2b};
use egui_dock::{NodeIndex, SurfaceIndex};
use egui_plot::{Line, PlotBounds, PlotPoints};
use serde::{Deserialize, Serialize};

use crate::state::{DataInspectorState, SignalData, XAxisMode};

const DEFAULT_PLOT_WIDTH: f64 = 30.0;
const PLOT_MARGIN_PC: f64 = 0.01;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Pane {
    pub pane_id: u64,
}

impl Pane {
    pub fn new(pane_id: u64) -> Self {
        Pane { pane_id }
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut DataInspectorState,
        signals: &mut SignalData,
        link_x_translated: &mut bool,
    ) {
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

        egui_plot::Plot::new(format!("plot_{}", self.pane_id))
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_double_click_reset(false)
            .link_cursor("main", true, false)
            .link_axis(
                "main",
                state.link_x && state.x_axis_mode != XAxisMode::Fit,
                false,
            )
            .auto_bounds(Vec2b::FALSE)
            .show(ui, |plot_ui| {
                for (id, signal) in signals.signals().get_signals() {
                    if state
                        .signal_state
                        .get(id)
                        .unwrap()
                        .used_by_tile
                        .contains(&self.pane_id)
                    {
                        let points = signal
                            .time()
                            .iter()
                            .zip(signal.data().iter())
                            .map(|(t, v)| [*t, *v])
                            .collect::<PlotPoints>();

                        plot_ui.line(Line::new(points));
                    }
                }

                let time_span = signals.time_span();
                // Plot mode transformations
                match state.x_axis_mode {
                    XAxisMode::Follow => {
                        let bounds = plot_ui.plot_bounds();
                        let dx = time_span.map(|s| s[1]).unwrap_or(DEFAULT_PLOT_WIDTH)
                            - bounds.max()[0]
                            + bounds.width() * PLOT_MARGIN_PC;

                        // To avoid artifacts, only one plot per frame can perform the translation when axis are linked
                        if !(state.link_x && *link_x_translated) {
                            plot_ui.translate_bounds(Vec2 {
                                x: dx as f32,
                                y: 0.0,
                            });
                            *link_x_translated = true;
                        }
                    }

                    XAxisMode::Fit => {
                        let bounds = plot_ui.plot_bounds();
                        let bounds = PlotBounds::from_min_max(
                            [
                                time_span.map(|s| s[0]).unwrap_or(0.0)
                                    - bounds.width() * PLOT_MARGIN_PC,
                                bounds.min()[1],
                            ],
                            [
                                time_span.map(|s| s[1]).unwrap_or(DEFAULT_PLOT_WIDTH)
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
                    }

                    if pointer_down {
                        let mut pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                        if state.x_axis_mode != XAxisMode::Free {
                            pointer_translate.x = 0.0;
                        }

                        plot_ui.translate_bounds(pointer_translate);
                    }
                }
            });
    }
}

pub struct TabViewer<'a> {
    state: &'a mut DataInspectorState,
    signals: &'a mut SignalData,

    link_x_translated: bool,

    pub added_nodes: Vec<(SurfaceIndex, NodeIndex)>,
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
    type Tab = Pane;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("Tab {}", tab.pane_id).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(ui, self.state, self.signals, &mut self.link_x_translated);
    }

    fn on_add(&mut self, surface: egui_dock::SurfaceIndex, node: egui_dock::NodeIndex) {
        self.added_nodes.push((surface, node));
    }
}
