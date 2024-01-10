use std::ops::RangeInclusive;

use crate::framehistory::FrameHistory;
use crate::layout::signallist::SignalListUI;
use crate::state::{DataInspectorState, SignalData, XAxisMode};
use rust_data_inspector_signals::Signals;

use egui::{Frame, ScrollArea};

pub struct DataInspector {
    signals: SignalData,
    state: DataInspectorState,
    frame_history: FrameHistory,
}

impl DataInspector {
    /// Called once before the first frame.
    #[allow(unused)]
    pub fn run(cc: &eframe::CreationContext<'_>, signals: Signals) -> Self {
        let state = DataInspectorState::new(&signals);

        let mut plotter_app = DataInspector {
            signals: SignalData::new(signals),
            state,
            frame_history: FrameHistory::default(),
        };

        plotter_app
    }
}

impl eframe::App for DataInspector {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.signals.signals.update();

        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            ui.horizontal(|ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Reset memory & close").clicked() {
                            ui.ctx().memory_mut(|mem| *mem = Default::default());
                            // self.invalidate_storage = true;
                            // frame.close()
                        }
                        if ui.button("Quit").clicked() {
                            // frame.close();
                        }
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.state.plot_x_width)
                            .speed(1.0)
                            .suffix(" s")
                            .clamp_range(RangeInclusive::new(0.001f64, std::f64::INFINITY)),
                    );
                    ui.label("Width:");

                    ui.selectable_value(&mut self.state.x_axis_mode, XAxisMode::Follow, "Follow");
                    ui.selectable_value(&mut self.state.x_axis_mode, XAxisMode::Fit, "Fit");
                    ui.selectable_value(&mut self.state.x_axis_mode, XAxisMode::Free, "Free");
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            SignalListUI::new().ui(ui, &self.signals, &mut self.state);
            // match self.plot_layout.tree.find_active_focused() {
            //     Some((_, tab)) => {
            //         SignalList::new().ui(ui, &mut self.signals, &mut tab.signals);
            //     }
            //     None => {
            //         SignalList::new().ui(
            //             ui,
            //             &mut self.signals,
            //             &mut TabSignals::new(),
            //         );
            //     }
            // }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                self.frame_history.ui(ui);
                ui.label(format!("FPS: {}", self.frame_history.fps()));
                // ui.label(format!("Num points: {}", self.num_points));
            });
        });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.label(format!("{:#?}", self.signals.signal_tree));
                });
                // The central panel the region left after adding TopPanel's and SidePanel's
                // self.num_points = 1usize;

                // self.plot_layout.ui(ui, &self.signals);
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, "plot_layout", &self.plot_layout);
        // eframe::set_value(storage, "invalidate_storage", &self.invalidate_storage);
    }
}
