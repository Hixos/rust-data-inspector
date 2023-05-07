use egui::plot::{Line, PlotBounds, PlotPoint, PlotPoints};
use egui_dock::NodeIndex;

use crate::{
    signal::Signal,
    util::plothelper::{AxisBounds, PlotHelper},
    widget::{RTPlot, LinkedAxisGroup},
    SignalGroup,
    
};
use std::collections::HashSet;

pub struct PlotLayout {
    pub tree: egui_dock::Tree<PlotTab>,
    pub settings: PlotSettings,

    tab_counter: usize,
}

#[derive(Clone)]
pub struct PlotSettings {
    pub real_time: bool,
    pub window_length: f64,
    pub link_group: LinkedAxisGroup,
}

impl Default for PlotSettings {
    fn default() -> Self {
        PlotSettings {
            real_time: true,
            window_length: 60f64,
            link_group: LinkedAxisGroup::new(true),
        }
    }
}

impl PlotLayout {
    pub fn new() -> Self {
        let mut tree = egui_dock::Tree::new(vec![PlotTab::new(1)]);
        tree.set_focused_node(NodeIndex::root());
        
        PlotLayout {
            tree,
            settings: PlotSettings::default(),
            tab_counter: 2usize,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, signals: &SignalGroup) {
        let mut style = egui_dock::Style::from_egui(ui.style().as_ref());
        style.show_add_buttons = true;
        let mut tab_viewer = PlotTabViewer::new(self.settings.clone(), signals);

        egui_dock::DockArea::new(&mut self.tree)
            .style(style)
            .show_inside(ui, &mut tab_viewer);

        match tab_viewer.add_tab {
            Some(index) => {
                let test = &mut self.tree[index];
                test.append_tab(PlotTab::new(self.tab_counter));
                self.tab_counter += 1;
            }
            None => {}
        }
    }
}

struct PlotTabViewer<'a> {
    add_tab: Option<egui_dock::NodeIndex>,
    settings: PlotSettings,
    signals: &'a SignalGroup,
}

impl<'a> PlotTabViewer<'a> {
    pub fn new(settings: PlotSettings, signals: &'a SignalGroup) -> Self {
        PlotTabViewer {
            add_tab: None,
            settings,
            signals,
        }
    }
}

impl<'a> egui_dock::TabViewer for PlotTabViewer<'a> {
    type Tab = PlotTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let mut plot = RTPlot::new(tab.index.to_string());

        if self.settings.real_time {
            let t = self.signals.current_timestamp().unwrap_or(0f64);

            plot.set_x_bounds(AxisBounds {
                min: t - self.settings.window_length,
                max: t,
            });
        }

        if self.settings.link_group.link_x {
            plot.link_axis(self.settings.link_group.clone());
        }

        plot.show(ui, |plot_ui| {
            let bounds = plot_ui.plot_bounds();

            let screen_rect = PlotHelper::get_screen_bounds(plot_ui);

            for signal in &tab.signals {
                plot_ui.line(PlotTab::get_line(
                    signal,
                    self.signals,
                    &bounds,
                    &screen_rect,
                ));
            }
        });

    }

    fn on_add(&mut self, node: egui_dock::NodeIndex) {
        self.add_tab = Some(node);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab.title()).into()
    }
}

pub struct PlotTab {
    pub signals: HashSet<String>,

    index: usize,
}

impl PlotTab {
    pub fn new(index: usize) -> Self {
        PlotTab {
            index,
            signals: HashSet::new(),
        }
    }

    pub fn title(&self) -> String {
        ["Tab".to_string(), self.index.to_string()].join(" ")
    }
}

impl PlotTab {
    pub fn get_line(
        signal_key: &String,
        signal_group: &SignalGroup,
        bounds: &PlotBounds,
        screen_bounds: &egui::Rect,
    ) -> Line {
        let signal = signal_group.get_signal(signal_key.clone());

        match signal {
            Some(signal) => {
                Line::new(Self::get_visible_points(signal, bounds, screen_bounds)).name(signal_key)
            }
            None => Line::new(PlotPoints::new(vec![])),
        }
    }

    fn get_visible_points(
        signal: &Signal,
        plot_bounds: &PlotBounds,
        screen_bounds: &egui::Rect,
    ) -> PlotPoints {
        // Don't plot every single point on the signal, but perform downsampling in order to improve performance
        // This can be further improved by
        //   - using binary to find the first point instead of linear search
        //   - using informtion from the previous frame to start the search since the starting points will probably be very close

        if signal.data().len() > 0 {
            // Goal is only having at most N points per pixel + 1 point outside the horizontal bounds for each side
            let mut points: Vec<PlotPoint> = Vec::with_capacity(screen_bounds.width() as usize + 2);
            // Minimum distance between plot points
            let ppp = plot_bounds.width() / screen_bounds.width() as f64 / 5f64;

            let mut first_index = 0;

            for (i, t) in signal.time().iter().enumerate().rev() {
                if *t < plot_bounds.min()[0] {
                    first_index = i;
                    break;
                }
            }
            points.push(PlotPoint {
                x: signal.time()[first_index],
                y: signal.data()[first_index],
            });

            for i in first_index + 1..signal.time().len() {
                let x = &signal.time()[i];
                if *x <= plot_bounds.max()[0] {
                    if x - points.last().unwrap().x >= ppp {
                        points.push(PlotPoint {
                            x: signal.time()[i],
                            y: signal.data()[i],
                        })
                    }
                } else {
                    // Insert first point outside of bounds to properly visualize line
                    points.push(PlotPoint {
                        x: signal.time()[i],
                        y: signal.data()[i],
                    });
                    break;
                }
            }

            PlotPoints::Owned(points)
        } else {
            PlotPoints::Owned(vec![])
        }
    }
}
