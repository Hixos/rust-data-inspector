use anyhow::{bail, anyhow, Result};
use clap::Parser;
use rust_data_inspector::datainspector::DataInspector;
use rust_data_inspector_signals::{PlotSignalProducer, PlotSignalSample, PlotSignals};
use std::{
    fs::File,
    io::{self, prelude::*, BufRead, BufReader, Read},
    path::Path,
    thread::spawn,
    time::Instant,
};
#[derive(Parser, Debug)]
#[command(name = "Rust Data Inspector")]
#[command(author = "Luca Erbetta <luca.erbetta105@gmail.com")]
#[command(version = "1.0")]
#[command(about = "Plot everything, from everywhere, all at once", long_about = None)]
struct Cli {
    #[arg(short, long)]
    pipe: bool,

    #[arg(short, long)]
    file: Option<String>,

    #[arg(long)]
    columns: Vec<String>,

    #[arg(short, long, default_value_t=String::from(","))]
    separator: String,

    #[arg(short, long)]
    time_index: Option<usize>,

    #[arg(short, long)]
    real_time: bool,

    #[arg(short, long)]
    count: Option<usize>,
}

fn read_lines<P>(path: P) -> io::Result<io::Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let reader = BufReader::new(File::open(path)?);
    Ok(reader.lines())
}

struct CSVPlotter {
    buf: Box<dyn BufRead + Send>,
    columns: Vec<String>,
    senders: Vec<PlotSignalProducer>,
    separator: String,
    time_index: Option<usize>,
    real_time: bool,
    max_lines: Option<usize>,
}

impl CSVPlotter {
    fn with_signals<R: BufRead + Send + 'static>(
        signals: &mut PlotSignals,
        mut reader: R,
        mut columns: Vec<String>,
        separator: String,
        time_index: Option<usize>,
        real_time: bool,
        max_lines: Option<usize>,
    ) -> Result<Self> {
        let mut header = String::default();

        reader.read_line(&mut header)?;

        let header = header.trim();

        if columns.is_empty() {
            columns = header
                .split(&separator)
                .map(|s| ["/", s].join(""))
                .collect();
        }

        let senders = columns
            .iter()
            .map(|c| signals.add_signal(c).expect("Error adding signal").1)
            .collect();

        if let Some(time_index) = time_index {
            if time_index >= columns.len() {
                bail!(
                    "Time index '{}' is out of bounds. Number of columns: {}",
                    time_index,
                    columns.len()
                );
            }
        }

        Ok(CSVPlotter {
            buf: Box::new(reader),
            columns,
            senders,
            separator,
            time_index,
            real_time,
            max_lines,
        })
    }

    fn plot_lines(&mut self) -> Result<()> {
        let mut step = 0usize;
        let start_time = Instant::now();

        let mut line = String::new();

        let mut last_time: f64 = 0.0;
        loop {
            step += 1;
            line.clear();
            self.plot_line(&mut line, start_time, step, &mut last_time)?;
        }
    }

    fn plot_line(
        &mut self,
        line: &mut String,
        start_time: Instant,
        step: usize,
        last_time: &mut f64,
    ) -> Result<usize> {
        let len = self.buf.read_line(line)?;

        let line = line.trim();

        if len > 0 {
            let values: Vec<f64> = line
                .split(&self.separator)
                .map(|c| c.parse::<f64>().map_err(|_| anyhow!("Error parsing string: '{c}'")))
                .collect::<Result<Vec<_>>>()?;

            let time = if let Some(time_index) = self.time_index {
                *values.get(time_index).unwrap()
            } else if self.real_time {
                (Instant::now() - start_time).as_secs_f64()
            } else {
                step as f64
            };

            if time <= *last_time {
                // Skipt this line
                return Ok(len);
            }

            *last_time = time;

            values.into_iter().enumerate().for_each(|(i, value)| {
                self.senders
                    .get_mut(i)
                    .unwrap()
                    .send(PlotSignalSample { time, value })
                    .unwrap();
            });
        }
        Ok(len)
    }
}

fn set_thread_panic_hook() {
    use std::{
        panic::{set_hook, take_hook},
        process::exit,
    };
    let orig_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        exit(1);
    }));
}

fn main() -> Result<()> {
    set_thread_panic_hook();
    let cli = Cli::parse();

    let mut signals = PlotSignals::default();

    let input: Box<dyn BufRead + Send + 'static> = if let Some(file) = cli.file {
        Box::new(BufReader::new(File::open(file)?))
    } else {
        let stdin = io::stdin();
        Box::new(BufReader::new(stdin))
    };

    let mut csvplotter = CSVPlotter::with_signals(
        &mut signals,
        input,
        cli.columns,
        cli.separator,
        cli.time_index,
        cli.real_time,
        cli.count,
    )?;

    spawn(move || {
        csvplotter.plot_lines().expect("Error readiung csv file");
    });

    DataInspector::run_native("Rust Data Inspector", signals).unwrap();
    Ok(())
}
