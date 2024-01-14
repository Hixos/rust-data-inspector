use crate::framehistory::FrameHistory;
use crate::layout::signallist::SignalListUI;
use crate::layout::tabs::{Tab, TabViewer};
use crate::state::{DataInspectorState, SignalData, TabState, XAxisMode};
use eframe::NativeOptions;
use egui_dock::{DockArea, Style};
use rust_data_inspector_signals::Signals;

use egui::Frame;

pub struct DataInspector {
    signals: SignalData,
    state: DataInspectorState,
    tab_state: TabState,
    frame_history: FrameHistory,
    reset: bool,
}

impl DataInspector {
    pub fn run_native(app_name: &str, signals: Signals) -> Result<(), eframe::Error>{
        eframe::run_native(
            app_name,
            NativeOptions::default(),
            Box::new(|cc| Box::new(DataInspector::run(cc, signals))),
        )
    }

    /// Called once before the first frame.
    #[allow(unused)]
    pub fn run(cc: &eframe::CreationContext<'_>, signals: Signals) -> Self {
        // Load from storage, if available
        let (state, tab_state) = if let Some(storage) = cc.storage {
            let state = DataInspectorState::from_storage(storage, &signals);
            let tab_state = eframe::get_value::<TabState>(storage, "tab_state");

            if let (Some(state), Some(tab_state)) = (state, tab_state) {
                (state, tab_state)
            } else {
                let mut state = DataInspectorState::new(&signals);
                let mut tab_state = TabState::default();

                (state, tab_state)
            }
        } else {
            let mut state = DataInspectorState::new(&signals);
            let mut tab_state = TabState::default();

            (state, tab_state)
        };

        DataInspector {
            signals: SignalData::new(signals),
            state,
            frame_history: FrameHistory::default(),
            tab_state,
            reset: false,
        }
    }

    fn reset(&mut self) {
        self.state = DataInspectorState::new(self.signals.signals());
        self.tab_state = TabState::default();
        self.frame_history = FrameHistory::default();
        self.reset = false;
    }
}

impl eframe::App for DataInspector {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.reset {
            ctx.memory_mut(|mem| *mem = Default::default());

            self.reset();

            if let Some(storage) = frame.storage_mut() {
                self.save(storage);
            }
        }

        self.signals.update();

        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

        // Find last interfacted pane
        if let Some((_, tab)) = self.tab_state.tree.find_active_focused() {
            self.state.selected_pane = tab.pane_id;
        }

        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Reset").clicked() {
                            self.reset = true;
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.state.x_axis_mode, XAxisMode::Follow, "Follow");
                    ui.selectable_value(&mut self.state.x_axis_mode, XAxisMode::Fit, "Fit");
                    ui.selectable_value(&mut self.state.x_axis_mode, XAxisMode::Free, "Free");
                    ui.label("Plot mode:");

                    ui.toggle_value(&mut self.state.link_x, "Link X");
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            SignalListUI::new().ui(ui, &self.signals, &mut self.state);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                self.frame_history.ui(ui);
                ui.label(format!("FPS: {}", self.frame_history.fps()));

                ui.label(format!("Active tile: {:X}", self.state.selected_pane));
                ui.label(format!("Signal bounds: {:?}", self.signals.time_span()));
            });
        });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let mut tabviewer = TabViewer::new(&mut self.state, &mut self.signals);
                let show_close_button = self.tab_state.tree.iter_all_tabs().count() > 1;
                DockArea::new(&mut self.tab_state.tree)
                    .show_add_buttons(true)
                    .show_close_buttons(show_close_button)
                    .style(Style::from_egui(ctx.style().as_ref()))
                    .show_inside(ui, &mut tabviewer);

                for (surface, node) in tabviewer.added_nodes.drain(..) {
                    self.tab_state
                        .tree
                        .set_focused_node_and_surface((surface, node));
                    self.tab_state
                        .tree
                        .push_to_focused_leaf(Tab::new(self.tab_state.tab_counter));
                    self.tab_state.tab_counter += 1;
                }
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.state.to_storage(storage);
        eframe::set_value(storage, "tab_state", &self.tab_state);
    }
}
