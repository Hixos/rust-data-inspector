use egui::Frame;

use crate::framehistory::FrameHistory;
use crate::layout::{PlotLayout, SignalList, XAxisMode};
use crate::signal_group::SignalGroup;
use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PlotterApp {
    signals: Arc<Mutex<SignalGroup>>,
    frame_history: FrameHistory,
    num_points: usize,
    plot_layout: PlotLayout,
    invalidate_storage: bool,
    // logic_thread: JoinHandle<>
}

impl PlotterApp {
    /// Called once before the first frame.
    #[allow(unused)]
    pub fn start<F>(cc: &eframe::CreationContext<'_>, app_logic: F) -> Self
    where
        F: FnOnce(Arc<Mutex<SignalGroup>>) + Send + Clone + 'static,
    {
        let mut plotter_app = PlotterApp {
            signals: Arc::new(Mutex::new(SignalGroup::new())),
            frame_history: FrameHistory::default(),
            num_points: 0,
            plot_layout: PlotLayout::new(),
            invalidate_storage: false,
        };

        if cc.storage.is_some() {
            let invalidate = eframe::get_value::<bool>(cc.storage.unwrap(), "invalidate_storage");

            if !invalidate.unwrap_or(false) {
                let pl = eframe::get_value::<PlotLayout>(cc.storage.unwrap(), "plot_layout");

                if pl.is_some() {
                    plotter_app.plot_layout = pl.unwrap();
                }
            }
        }

        let signals = plotter_app.signals.clone();

        // Start app logic thread
        thread::spawn(move || app_logic(signals));

        plotter_app
    }
}

impl eframe::App for PlotterApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.signals.lock().unwrap().receive();
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
                            self.invalidate_storage = true;
                            frame.close()
                        }
                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.plot_layout.settings.window_length)
                            .speed(1.0)
                            .suffix(" s")
                            .clamp_range(RangeInclusive::new(0.001f64, std::f64::INFINITY)),
                    );
                    ui.label("Width:");

                    ui.selectable_value(
                        &mut self.plot_layout.settings.x_axis_mode,
                        XAxisMode::FOLLOW,
                        "Follow",
                    );
                    ui.selectable_value(
                        &mut self.plot_layout.settings.x_axis_mode,
                        XAxisMode::FIT,
                        "Fit",
                    );
                    ui.selectable_value(
                        &mut self.plot_layout.settings.x_axis_mode,
                        XAxisMode::FREE,
                        "Free",
                    );
                    ui.toggle_value(&mut self.plot_layout.settings.link_group.link_x, "Link X");
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            match self.plot_layout.tree.find_active_focused() {
                Some((_, tab)) => {
                    SignalList::new().ui(ui, &mut self.signals.lock().unwrap(), &mut tab.signals);
                }
                None => {
                    SignalList::new().ui(
                        ui,
                        &mut self.signals.lock().unwrap(),
                        &mut HashSet::new(),
                    );
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                self.frame_history.ui(ui);
                ui.label(format!("FPS: {}", self.frame_history.fps()));
                ui.label(format!("Num points: {}", self.num_points));
            });
        });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                self.num_points = 1usize;

                self.plot_layout.ui(ui, &self.signals.lock().unwrap());
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "plot_layout", &self.plot_layout);
        eframe::set_value(storage, "invalidate_storage", &self.invalidate_storage);
    }
}
