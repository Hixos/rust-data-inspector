#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use rand::Rng;
use std::f64::consts::PI;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use rust_data_inspector_signals::{PlotSignals,  PlotSignalSample};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use rust_data_inspector::DataInspector;

    let mut signals = PlotSignals::default();

    let start = Instant::now();
    let mut rng = rand::thread_rng();
    let mut add_signal = |name: &str| {
        new_signal_producer(
            &mut signals,
            name,
            rng.gen::<f64>() * 10.0 + 5.0,
            rng.gen(),
            rng.gen::<f64>() * PI * 2.0,
            rng.gen::<f32>() * 60f32 + 2f32,
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

    DataInspector::run_native("plotter", signals)
}

pub fn new_signal_producer(
    signals: &mut PlotSignals,
    name: &str,
    a: f64,
    f: f64,
    phi: f64,
    rate: f32,
    start_time: Option<Instant>,
) -> JoinHandle<()> {
    let (_, mut sample_sender) = signals.add_signal(name).unwrap();

    thread::spawn(move || {
        let period_ms = u64::max((1000f32 / rate) as u64, 1);

        let start = start_time.unwrap_or(Instant::now());

        loop {
            let t = Instant::now() - start;
            let t = t.as_secs_f64();

            let y = a * f64::sin(2f64 * PI * f * t + phi);

            let res = sample_sender.send(PlotSignalSample { time: t, value: y });

            if res.is_ok() {
                thread::sleep(Duration::from_millis(period_ms))
            } else {
                break;
            }
        }
    })
}
