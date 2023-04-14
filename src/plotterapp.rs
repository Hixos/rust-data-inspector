use std::collections::HashSet;
use std::sync::mpsc::Receiver;

use egui::Rect;

use crate::layout::{PlotLayout, SignalList};
use crate::signal_group::{SignalGroup, SignalHandle};
use crate::{framehistory::FrameHistory, widget::RTPlot};

pub struct PlotterApp {
    signals: SignalGroup,
    frame_history: FrameHistory,
    num_points: usize,
    test: bool,
    plot_layout: PlotLayout
}

impl PlotterApp {
    /// Called once before the first frame.
    #[allow(unused)]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        new_signal_receiver: Receiver<SignalHandle>,
    ) -> Self {
        return PlotterApp {
            signals: SignalGroup::new(new_signal_receiver),
            frame_history: FrameHistory::default(),
            num_points: 0,
            test: false,
            plot_layout: PlotLayout::new()
        };
    }
}

impl eframe::App for PlotterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.signals.update();

        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), _frame.info().cpu_usage);

        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            ui.horizontal(|ui| {
                // egui::menu::bar(ui, |ui| {
                //     ui.menu_button("File", |ui| {
                //         if ui.button("Quit").clicked() {
                //             _frame.close();
                //         }
                //     });
                // });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reset memory").clicked() {
                        ui.ctx().memory_mut(|mem| *mem = Default::default());
                    }

                    if ui.selectable_label(self.test, "Prova 123").clicked() {
                        // self.test = !self.test;
                    }

                    ui.label("a1234");
                    ui.label("b1234");
                    ui.label("c1234");
                    ui.label("d1234");
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            // SignalList::new().ui(ui, sign);

            
            match self.plot_layout.tree.find_active_focused() {
                Some((_, tab)) => {
                    SignalList::new().ui(ui, &self.signals, &mut tab.signals);

                }
                None => {
                    SignalList::new().ui(ui, &self.signals, &mut HashSet::new());
                }
            }


            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                self.frame_history.ui(ui);
                ui.label(format!("FPS: {}", self.frame_history.fps()));
                ui.label(format!("Num points: {}", self.num_points));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            self.num_points = 1usize;
            let sig = self.signals.get_signal("a/b/s1");

            self.plot_layout.ui(ui, &self.signals);

            // RTPlot::new("rt_plot").show(ui, sig.unwrap()); // Panic for now
        });
    }
}
