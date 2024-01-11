use std::ops::RangeInclusive;

use crate::framehistory::FrameHistory;
use crate::layout::signallist::SignalListUI;
use crate::layout::tiles::{Pane, TilesBehavior};
use crate::state::{DataInspectorState, SignalData, XAxisMode};
use egui_tiles::Tile;
use rust_data_inspector_signals::Signals;

use egui::{Frame, ScrollArea};

pub struct DataInspector {
    signals: SignalData,
    state: DataInspectorState,
    tile_tree: egui_tiles::Tree<Pane>,
    frame_history: FrameHistory,
}

impl DataInspector {
    /// Called once before the first frame.
    #[allow(unused)]
    pub fn run(cc: &eframe::CreationContext<'_>, signals: Signals) -> Self {
        let mut state = DataInspectorState::new(&signals);
        let tile_tree = create_tile_tree(&mut state);
        DataInspector {
            signals: SignalData::new(signals),
            state,
            frame_history: FrameHistory::default(),
            tile_tree,
        }
    }
}

fn create_tile_tree(state: &mut DataInspectorState) -> egui_tiles::Tree<Pane> {
    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    tabs.push({
        let child = tiles.insert_pane(Pane {
            id: state.get_tile_id_and_increment(),
        });
        tiles.insert_horizontal_tile([child].to_vec())
    });

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("display_tree", root, tiles)
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
            ui.label(format!("Active tile: {:X}", self.state.selected_tile));
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
                let mut behavior = TilesBehavior::new(&mut self.state, &mut self.signals);
                self.tile_tree.ui(&mut behavior, ui);

                let close_tab = behavior.close_tab;

                if let Some(tile_id) = behavior.add_child_to.take() {
                    let id = self.state.get_tile_id_and_increment();
                    self.state.selected_tile = id;

                    let new_child = self.tile_tree.tiles.insert_pane(Pane { id });

                    if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                        self.tile_tree.tiles.get_mut(tile_id)
                    {
                        tabs.add_child(new_child);
                        tabs.set_active(new_child);
                    }
                }

                if let Some(tile_id) = close_tab {
                    // When there is only one pane left, we have the pane tile + its container
                    // We don't want to delete it
                    if self.tile_tree.tiles.len() > 2 {
                        if let Some(Tile::Pane(pane)) = self.tile_tree.tiles.get(tile_id) {
                            for signal in self.state.signal_state.values_mut() {
                                if signal.used_by_tile.contains(&pane.id) {
                                    signal.used_by_tile.remove(&pane.id);
                                }
                            }
                        }

                        self.tile_tree.remove_recursively(tile_id);
                    }
                }
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let _ = storage;
        // eframe::set_value(storage, "plot_layout", &self.plot_layout);
        // eframe::set_value(storage, "invalidate_storage", &self.invalidate_storage);
    }
}
