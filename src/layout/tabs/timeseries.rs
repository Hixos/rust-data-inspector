use std::{
    collections::HashMap,
    ops::{Range, RangeInclusive},
};

use downsample_rs::lttb_with_x;
use egui::{Event, Vec2, Vec2b};
use egui_plot::{Legend, Line, PlotBounds, PlotPoints};
use rust_data_inspector_signals::PlotSignalID;
use serde::{Deserialize, Serialize};

use crate::{
    state::{DataInspectorState, SignalData, XAxisMode},
    utils::downsampling::{decimate, find_visible_index_range, DownsamplingMethod},
};

const DEFAULT_PLOT_WIDTH: f64 = 30.0;
const PLOT_MARGIN_PC: f64 = 0.01;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TimeSeriesTab {
    tab_id: u64,
    #[serde(skip)]
    cache: HashMap<PlotSignalID, SignalPlotCache>,
}

#[derive(Debug, Default, Clone)]
struct SignalPlotCache {
    last_index_range: Range<usize>,
}

impl TimeSeriesTab {
    pub fn new(tab_id: u64) -> Self {
        TimeSeriesTab {
            tab_id,
            cache: HashMap::new(),
        }
    }

    pub fn ui(
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

        let plot_response = egui_plot::Plot::new(format!("plot_{}", self.tab_id))
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
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let plot_rect_width = plot_ui
                    .screen_from_plot(plot_ui.plot_bounds().max().into())
                    .x as usize
                    - plot_ui
                        .screen_from_plot(plot_ui.plot_bounds().min().into())
                        .x as usize;

                let x_bounds = RangeInclusive::new(
                    plot_ui.plot_bounds().min()[0],
                    plot_ui.plot_bounds().max()[0],
                );

                for (id, signal) in signals.signals().get_signals() {
                    if let Some(sig_state) = state.signal_state.get(id) {
                        if sig_state.used_by_tile.contains(&self.tab_id) {
                            let range = find_visible_index_range(
                                signal,
                                &x_bounds,
                                self.cache.get(id).map(|c| c.last_index_range.clone()),
                            );

                            if let Some(range) = range {
                                self.cache.insert(
                                    *id,
                                    SignalPlotCache {
                                        last_index_range: range.clone(),
                                    },
                                );
                                let time = signal.time().get(range.clone()).unwrap();
                                let data = signal.data().get(range.clone()).unwrap();
                                let points = Self::downsample(
                                    time,
                                    data,
                                    plot_rect_width,
                                    state.downsample_mode,
                                );

                                plot_ui.line(
                                    Line::new(points).color(sig_state.color).name(signal.name()),
                                );
                            }
                        }
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
                };

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

        let bounds = plot_response.transform.bounds();
        let visible_range = RangeInclusive::new(bounds.min()[0], bounds.max()[0]);
        if state.link_x {
            if !*link_x_translated {
                state.visible_range = visible_range;
            }
        } else if plot_response.response.hovered() {
            println!("Changed");
            state.visible_range = visible_range;
        }

        *link_x_translated = true;
    }

    fn downsample(
        time: &[f64],
        data: &[f64],
        rec_width: usize,
        mode: DownsamplingMethod,
    ) -> PlotPoints {
        let indices = match mode {
            DownsamplingMethod::Decimation => decimate(time, rec_width * 2),
            DownsamplingMethod::Lttb => lttb_with_x(time, data, rec_width * 2),
        };

        indices
            .into_iter()
            .map(|i| [time[i], data[i]])
            .collect::<PlotPoints>()
    }
}
