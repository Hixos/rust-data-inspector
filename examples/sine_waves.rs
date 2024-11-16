#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use rand::Rng;
use rust_data_inspector::{DataInspector, DataInspectorAPI};
use rust_data_inspector_signals::{PlotSignalSample, PlotSignals};
use std::f64::consts::PI;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use std::sync::atomic::AtomicBool;

    let mut signals = PlotSignals::default();

    let mut rng = rand::thread_rng();
    let pause = Arc::new(AtomicBool::new(false));

    let mut add_signal = |name: &str| {
        new_signal_producer(
            &mut signals,
            name,
            rng.gen::<f64>() * 10.0 + 5.0,
            rng.gen(),
            rng.gen::<f64>() * PI * 2.0,
            rng.gen::<f32>() * 60f32 + 2f32,
            pause.clone(),
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

    DataInspector::run_native(
        "plotter",
        signals,
        Some(move |ui: &mut egui::Ui, _: &mut DataInspectorAPI| {
            let mut p = pause.load(Relaxed);

            ui.toggle_value(&mut p, "Pause");

            pause.store(p, Relaxed);
        }),
    )
}

pub fn new_signal_producer(
    signals: &mut PlotSignals,
    name: &str,
    a: f64,
    f: f64,
    phi: f64,
    rate: f32,
    pause: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let (_, sample_sender) = signals.add_signal(name).unwrap();

    thread::spawn(move || {
        let period_ms = u64::max((1000f32 / rate) as u64, 1);

        let mut t = 0.0;

        loop {
            let y = a * f64::sin(2f64 * PI * f * t + phi);
            let res = if !pause.load(Relaxed) {
                let res = sample_sender.send(PlotSignalSample { time: t, value: y });
                t += period_ms as f64 / 1000.0;
                res
            } else {
                Ok(())
            };

            if res.is_ok() {
                thread::sleep(Duration::from_millis(period_ms))
            } else {
                break;
            }
        }
    })
}
