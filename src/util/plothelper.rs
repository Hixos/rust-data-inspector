use egui::plot::{PlotBounds, PlotPoint};
use serde::{Deserialize, Serialize};


pub struct PlotHelper{}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AxisBounds {
    pub min: f64,
    pub max: f64,
}

impl AxisBounds {
    pub fn from_x_bounds(bounds: PlotBounds) -> Self {
        Self { min: bounds.min()[0], max: bounds.max()[0] }
    }
}


impl PlotHelper {
    pub fn set_x_bounds(
        x_bounds: AxisBounds,
        bounds: egui::plot::PlotBounds,
    ) -> egui::plot::PlotBounds {
        PlotBounds::from_min_max(
            [x_bounds.min, bounds.min()[1]],
            [x_bounds.max, bounds.max()[1]],
        )
    }

    pub fn set_y_bounds(
        y_bounds: AxisBounds,
        bounds: egui::plot::PlotBounds,
    ) -> egui::plot::PlotBounds {
        PlotBounds::from_min_max(
            [bounds.min()[0], y_bounds.min],
            [bounds.max()[0], y_bounds.max],
        )
    }

    pub fn get_x_bounds(bounds: egui::plot::PlotBounds) -> AxisBounds {
        AxisBounds {
            min: bounds.min()[0],
            max: bounds.max()[0],
        }
    }

    pub fn get_y_bounds(bounds: egui::plot::PlotBounds) -> AxisBounds {
        AxisBounds {
            min: bounds.min()[1],
            max: bounds.max()[1],
        }
    }

    pub fn get_screen_bounds(plot_ui: &mut egui::plot::PlotUi) -> egui::Rect {
        let bounds = plot_ui.plot_bounds();
        // Y axis min/max are swapped since screen-space Y axis is positive downwards
        let topleft = PlotPoint::new(bounds.min()[0], bounds.max()[1]);
        let bottomright = PlotPoint::new(bounds.max()[0], bounds.min()[1]);

        egui::Rect {
            min: plot_ui.screen_from_plot(topleft),
            max: plot_ui.screen_from_plot(bottomright),
        }
    }
}