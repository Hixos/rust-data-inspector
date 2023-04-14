#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use plotter::new_signal_producer;
use rand::Rng;
use std::sync::mpsc::channel;
use std::time::Instant;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {

    let start = Instant::now();

    let (new_sig_sender, new_sig_receiver) = channel();

    let mut rng = rand::thread_rng();
    let mut add_signal = |name: &str| {
        use std::f64::consts::PI;

        let (_, shandle) = new_signal_producer(
            String::from(name),
            rng.gen::<f64>() * 10.0 + 5.0,
            rng.gen(),
            rng.gen::<f64>() * PI * 2.0,
            rng.gen::<f32>() * 100f32 + 2f32,
            Some(start),
        );

        new_sig_sender.send(shandle).unwrap(); // Panic on failure
    };

    add_signal("a/b/s1");
    add_signal("a/b/s2");
    add_signal("a/b/s3");
    add_signal("a/c/s1");
    add_signal("a/c/s2");
    add_signal("b/s1");
    add_signal("b/s2");
    add_signal("b/c/s1");
    add_signal("s1");
    add_signal("s2");

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
