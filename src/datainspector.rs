use crate::framehistory::FrameHistory;
use crate::layout::signallist::SignalListUI;
use crate::layout::tiles::{Pane, TilesBehavior};
use crate::state::{DataInspectorState, SignalData, TileState, XAxisMode};
use egui_tiles::Tile;
use rust_data_inspector_signals::Signals;

use egui::Frame;

pub struct DataInspector {
    signals: SignalData,
    state: DataInspectorState,
    tile_tree: egui_tiles::Tree<Pane>,
    frame_history: FrameHistory,
    reset: bool,
}

impl DataInspector {
    /// Called once before the first frame.
    #[allow(unused)]
    pub fn run(cc: &eframe::CreationContext<'_>, signals: Signals) -> Self {

        // Load from storage, if available
        let (state, tile_tree) = if let Some(storage) = cc.storage {
            let state = DataInspectorState::from_storage(storage, &signals);
            let tile_tree = eframe::get_value::<egui_tiles::Tree<Pane>>(storage, "tile_tree");

            if let (Some(state), Some(tile_tree)) = (state, tile_tree) {
                (state, tile_tree)
            } else {
                let mut state = DataInspectorState::new(&signals);
                let mut tile_tree = create_tile_tree(&mut state);

                (state, tile_tree)
            }
        } else {
            let mut state = DataInspectorState::new(&signals);
            let mut tile_tree = create_tile_tree(&mut state);

            (state, tile_tree)
        };

        DataInspector {
            signals: SignalData::new(signals),
            state,
            frame_history: FrameHistory::default(),
            tile_tree,
            reset: false,
        }
    }

    fn reset(&mut self) {
        self.state = DataInspectorState::new(self.signals.signals());
        self.tile_tree = create_tile_tree(&mut self.state);
        self.frame_history = FrameHistory::default();
        self.reset = false;
    }
}

fn create_tile_tree(state: &mut DataInspectorState) -> egui_tiles::Tree<Pane> {
    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    let pane_id = state.get_pane_id_and_increment();
    state.pane_state.insert(pane_id, TileState::default());

    tabs.push({
        let child = tiles.insert_pane(Pane { id: pane_id });
        tiles.insert_horizontal_tile([child].to_vec())
    });

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("display_tree", root, tiles)
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

        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

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

                ui.label(format!("Active tile: {:X}", self.state.selected_tile));
                ui.label(format!("Signal bounds: {:?}", self.signals.time_span()));
            });
        });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let mut behavior = TilesBehavior::new(&mut self.state, &mut self.signals);
                self.tile_tree.ui(&mut behavior, ui);

                let close_tab = behavior.close_tab;

                if let Some(tile_id) = behavior.add_child_to.take() {
                    let id = self.state.get_pane_id_and_increment();
                    self.state.pane_state.insert(id, TileState::default());
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
                            self.state.pane_state.remove(&pane.id);
                        }
                        self.tile_tree.remove_recursively(tile_id);
                    }
                }
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.state.to_storage(storage);
        eframe::set_value(storage, "tile_tree", &self.tile_tree);
    }
}
