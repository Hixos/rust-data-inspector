#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use plotter::new_signal_producer;
use std::time::Instant;
use std::sync::mpsc::channel;
    use std::{thread::JoinHandle};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).


    let mut handles: Vec<JoinHandle<()>> = vec![];
    let start = Instant::now();

    let (new_sig_sender, new_sig_receiver) = channel();

    for i in 0..2 {
        // let name = 
        let (thandle, shandle) = new_signal_producer(
            String::from("Signal ") + &i.to_string(),
            10f64,
            0.25f64,
            0.123,
            60f32,
            Some(start)
        );

        handles.push(thandle);
        new_sig_sender.send(shandle).unwrap(); // Panic on failure
    }

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Plotter",
        native_options,
        Box::new(|cc| Box::new(plotter::PlotterApp::new(cc, new_sig_receiver))),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(plotter::PlotterApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
