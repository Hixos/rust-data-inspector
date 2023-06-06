#![warn(clippy::all, rust_2018_idioms)]

mod plotterapp;
mod framehistory;
mod widget;
mod layout;
mod signal;
mod signal_group;
mod util;

pub use plotterapp::PlotterApp;
pub use framehistory::FrameHistory;
pub use signal_group::{SignalGroup, SignalHandle};
pub use signal::{Signal, SignalSample};

