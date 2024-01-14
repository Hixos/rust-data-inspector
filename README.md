# Rust Data Inspector

A flexible library to plot time series in real time.

![Rust Data Inspector](docs/screenshot.png "Rust Data Inspector")

## Features
- Flexible: Easily provide your own source for signals. Only requirement is that time is strictly monotonic
- Fast: can display signals with millions of samples effortlessy
- Customizable: Structure the UI to your liking through [egui_dock](https://github.com/Adanos020/egui_dock), select signal colors etc...


## Examples
See `examples/sine_waves.rs`