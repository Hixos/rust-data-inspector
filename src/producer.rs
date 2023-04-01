use super::signal::{Signal, SignalSample};
use super::signal_group::SignalHandle;

use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{f64::consts::PI, sync::mpsc, time::Instant};

pub fn new_signal_producer<S: Into<String>>(
    name: S,
    a: f64,
    f: f64,
    phi: f64,
    rate: f32,
    start_time: Option<Instant>,
) -> (JoinHandle<()>, SignalHandle) {
    let (sender, receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
        let period_ms = u64::max((1000f32 / rate) as u64, 1);

        let start = start_time.unwrap_or(Instant::now());

        loop {
            let t = Instant::now() - start;
            let t = t.as_secs_f64();

            let y = a * f64::sin(2f64 * PI * f * t + phi);

            let res = sender.send(SignalSample { t, y });

            if res.is_ok() {
                thread::sleep(Duration::from_millis(period_ms))
            } else {
                break;
            }
        }
    });

    return (
        handle,
        SignalHandle {
            signal: Signal::new(name),
            receiver,
        },
    );
}
