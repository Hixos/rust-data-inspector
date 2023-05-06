use egui::plot::{Line, PlotBounds, PlotPoint, PlotPoints};

use crate::signal::Signal;
use crate::SignalGroup;

pub struct PlotLogic {
    pub last_window: [f64; 2],
    pub signals: Vec<String>,
}

struct SignalPlotData {}

