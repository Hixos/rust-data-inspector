#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use rand::Rng;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use rust_data_inspector_signals::{Signals, Signal, SignalSample};
use rust_data_inspector::datainspector;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    let mut signals = Signals::default();

    let start = Instant::now();
    let mut rng = rand::thread_rng();
    let mut add_signal = |name: &str| {
        new_signal_producer(
            &mut signals,
            name,
            rng.gen::<f64>() * 10.0 + 5.0,
            rng.gen(),
            rng.gen::<f64>() * PI * 2.0,
            rng.gen::<f32>() * 100f32 + 2f32,
            Some(start),
        );
    };

    add_signal("/a/b/s1");
    add_signal("/a/b/s2");
    add_signal("/a/b/s3");
    add_signal("/a/c/s1");
    add_signal("/a/c/s2");
    add_signal("/b/s1");
    add_signal("/b/s2");
    add_signal("/b/c/s1");
    add_signal("/s1");
    add_signal("/s2");

    eframe::run_native(
        "Plotter",
        native_options,
        Box::new(|cc| Box::new(datainspector::DataInspector::run(cc, signals))),
    )
}

// when compiling to web using trunk.
// #[cfg(target_arch = "wasm32")]
// fn main() {
//     // Make sure panics are logged using `console.error`.
//     console_error_panic_hook::set_once();

//     // Redirect tracing to console.log and friends:
//     tracing_wasm::set_as_global_default();

//     let web_options = eframe::WebOptions::default();

//     wasm_bindgen_futures::spawn_local(async {
//         eframe::start_web(
//             "the_canvas_id", // hardcode it
//             web_options,
//             Box::new(|cc| Box::new(plotter::PlotterApp::new(cc))),
//         )
//         .await
//         .expect("failed to start eframe");
//     });
// }

pub fn new_signal_producer(
    signals: &mut Signals,
    name: &str,
    a: f64,
    f: f64,
    phi: f64,
    rate: f32,
    start_time: Option<Instant>,
) -> JoinHandle<()> {
    let (id, mut sample_sender) = signals.add_signal(name).unwrap();

    let handle = thread::spawn(move || {
        let period_ms = u64::max((1000f32 / rate) as u64, 1);

        let start = start_time.unwrap_or(Instant::now());

        loop {
            let t = Instant::now() - start;
            let t = t.as_secs_f64();

            let y = a * f64::sin(2f64 * PI * f * t + phi);

            let res = sample_sender.send(SignalSample { time: t, value: y });

            if res.is_ok() {
                thread::sleep(Duration::from_millis(period_ms))
            } else {
                break;
            }
        }
    });

    handle
}
