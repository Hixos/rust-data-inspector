#![warn(clippy::all, rust_2018_idioms)]

mod plotterapp;
mod producer;
mod framehistory;
mod widget;
mod layout;
mod signal;
mod signal_group;
// mod util;

pub use plotterapp::PlotterApp;
pub use producer::new_signal_producer;
pub use framehistory::FrameHistory;
pub use signal_group::{SignalGroup, SignalHandle};