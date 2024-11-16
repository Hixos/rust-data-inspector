use std::{collections::HashMap, ops::Range};

use egui::{CollapsingHeader, Frame, Stroke, Vec2b};
use egui_plot::{Legend, Line, PlotPoints};
use rust_data_inspector_signals::PlotSignalID;
use serde::{Deserialize, Serialize};

use crate::state::{DataInspectorState, SignalData};

const DEFAULT_PLOT_WIDTH: f64 = 30.0;
const PLOT_MARGIN_PC: f64 = 0.01;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct XYPlotTab {
    tab_id: u64,

    #[serde(skip)]
    cache: HashMap<PlotSignalID, XYPlotCache>,
}

#[derive(Debug, Clone, Default)]
struct XYPlotCache {
    last_index_range: Range<usize>,
}

impl XYPlotTab {
    pub fn new(tab_id: u64) -> Self {
        XYPlotTab {
            tab_id,
            cache: HashMap::new(),
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut DataInspectorState,
        signals: &mut SignalData,
    ) {
        let response = egui_plot::Plot::new(format!("plot_{}", self.tab_id))
            .auto_bounds(Vec2b::TRUE)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let mut selected_signals = Vec::<PlotSignalID>::new();

                for (id, _) in signals.signals().get_signals() {
                    if let Some(sig_state) = state.signal_state.get(id) {
                        if sig_state.used_by_tile.contains(&self.tab_id) {
                            selected_signals.push(*id);
                        }
                    }
                }

                if selected_signals.len() >= 2 {
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

                    let sig_x = signals.signals().get_signal(selected_signals[0]);
                    let sig_y = signals.signals().get_signal(selected_signals[1]);

                    let points: PlotPoints = sig_x
                        .data()
                        .iter()
                        .zip(sig_y.data().iter())
                        .map(|(&x, &y)| [x, y])
                        .collect();

                    plot_ui.line(Line::new(points));
                }
            });
        response.response.context_menu(|ui| {
            ui.label("Hello menu!");
        });
    }
}
