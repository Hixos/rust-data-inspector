use std::ops::RangeInclusive;

use crate::framehistory::FrameHistory;
use crate::layout::dock::MyTabs;
use crate::signal::{SignalGroup, SignalID};
use crate::utils::SimpleTree;
use egui::ahash::{HashMap, HashMapExt};
use egui::Frame;

pub struct DataInspector {
    frame_history: FrameHistory,
    signals: SignalGroup,

    config: Configuration,
    dock: MyTabs,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum XAxisMode {
    Fit,
    Follow,
    Free,
}

pub(crate) struct Configuration {
    x_axis_mode: XAxisMode,
    plot_x_width: f64,
}

#[derive(Default)]
pub(crate) struct SignalGUIData {}

pub(crate) struct SignalNode {
    pub path: String,
    pub signal: Option<SignalID>,
}

pub(crate) struct SignalState {
    signal_tree: SimpleTree<SignalNode>,
    data: HashMap<SignalID, SignalGUIData>,
}

impl SignalState {
    pub fn new(signals: SignalGroup) -> Self {
        let mut data: HashMap<SignalID, SignalGUIData> = HashMap::new();
        let mut signal_tree = SimpleTree::<SignalNode>::new(SignalNode {
            path: "/".to_owned(),
            signal: None,
        });

        for (id, signal) in signals.get_signals().iter() {
            data.insert(*id, SignalGUIData::default());

            let path = signal.path_elements();

            ()
        }

        SignalState { signal_tree, data }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            x_axis_mode: XAxisMode::Fit,
            plot_x_width: 60f64,
        }
    }
}

impl DataInspector {
    /// Called once before the first frame.
    #[allow(unused)]
    pub fn run(cc: &eframe::CreationContext<'_>, signals: SignalGroup) -> Self {
        let mut plotter_app = DataInspector {
            frame_history: FrameHistory::default(),
            signals: SignalGroup::default(),
            config: Configuration::default(),
            dock: MyTabs::new(),
        };

        plotter_app
    }
}

impl eframe::App for DataInspector {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.signals.update_signals();

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
                            frame.close()
                        }
                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.config.plot_x_width)
                            .speed(1.0)
                            .suffix(" s")
                            .clamp_range(RangeInclusive::new(0.001f64, std::f64::INFINITY)),
                    );
                    ui.label("Width:");

                    ui.selectable_value(&mut self.config.x_axis_mode, XAxisMode::Follow, "Follow");
                    ui.selectable_value(&mut self.config.x_axis_mode, XAxisMode::Fit, "Fit");
                    ui.selectable_value(&mut self.config.x_axis_mode, XAxisMode::Free, "Free");
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
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
                self.dock.ui(ui);
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
