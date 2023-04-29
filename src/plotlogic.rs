use std::default;

use egui::plot::{Line, PlotBounds, PlotPoint, PlotPoints};

use crate::signal::Signal;
use crate::SignalGroup;

pub struct GlobalPlotLogic {
    pub link_axes: bool,
    pub real_time: bool,
    pub current_timestamp: f64,
    pub window_length: f64,
}

impl Default for GlobalPlotLogic {
    fn default() -> Self {
        GlobalPlotLogic { link_axes: true, real_time: true, current_timestamp: 0f64, window_length: 60f64 }
    }
}

pub struct PlotLogic {
    pub last_window: [f64; 2],
    pub signals: Vec<String>,
}

struct SignalPlotData {}

impl PlotLogic {
    pub fn get_line<S: Into<String>>(
        signal_key: S,
        signal_group: SignalGroup,
        bounds: &PlotBounds,
        screen_bounds: &egui::Rect,
    ) -> Line {
        let signal = signal_group.get_signal(signal_key);

        match signal {
            Some(signal) => Line::new(Self::get_visible_points(signal, bounds, screen_bounds))
                .name(Into::<String>::into(signal_key)),
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
