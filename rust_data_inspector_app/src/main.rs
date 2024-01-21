use anyhow::{anyhow, Result};
use clap::Parser;
use rust_data_inspector::DataInspector;
use rust_data_inspector_signals::{PlotSampleSender, PlotSignalSample, PlotSignals};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
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
    file: Option<String>,

    #[arg(long)]
    columns: Vec<String>,

    #[arg(long)]
    col_hint: Option<usize>,

    #[arg(short, long, default_value_t=String::from(","))]
    separator: String,

    #[arg(short, long)]
    time_index: Option<usize>,

    #[arg(short, long)]
    real_time: bool,
}

struct CSVPlotter {
    reader: Box<dyn BufRead + Send>,
    line_sender: LineSender,
}

impl CSVPlotter {
    fn with_signals<R: BufRead + Send + 'static>(
        mut reader: R,
        mut _columns: Vec<String>,
        separator: String,
        time_col_index: Option<usize>,
        real_time: bool,
        hint_cols: Option<usize>,
    ) -> (Self, PlotSignals) {
        println!("- Waiting for valid data...");

        let (signals, senders, first_data) =
            Self::find_data(&mut reader, hint_cols, separator.clone());

        let mut line_sender = if let Some(time_col_index) = time_col_index {
            LineSender::new(TimeIndex::Column(time_col_index, None), senders, separator)
        } else if real_time {
            LineSender::new(TimeIndex::Generated(Instant::now()), senders, separator)
        } else {
            LineSender::new(TimeIndex::Counter(0), senders, separator)
        };

        let _ = line_sender.send_line(&first_data);

        (
            CSVPlotter {
                reader: Box::new(reader),
                line_sender,
            },
            signals,
        )
    }

    fn plot_lines(&mut self) {
        let mut buf = String::new();
        let mut index = 2; // Starts from 2 since we read the first line in find_data
        while let Ok(len) = self.reader.read_line(&mut buf) {
            if len == 0 {
                break;
            }
            if let Err(e) = self.line_sender.send_line(buf.as_str()) {
                println!("Error reading line {}: {}", index, e);
            }
            index += 1;
            buf.clear();
        }
    }

    fn find_data<R: BufRead + Send + 'static>(
        reader: &mut R,
        hint_cols: Option<usize>,
        separator: String,
    ) -> (PlotSignals, Vec<PlotSampleSender>, String) {
        enum State {
            FindHeader,
            CheckData,
        }

        let mut signals = PlotSignals::default();
        let mut producers: Vec<PlotSampleSender> = vec![];

        let mut state = State::FindHeader;
        let mut buf = String::new();

        let mut first_data: Option<String> = None;

        'line_loop: while let Ok(len) = reader.read_line(&mut buf) {
            if len == 0 {
                break 'line_loop;
            }
            let line = buf.trim();

            let mut success = false;
            match state {
                State::FindHeader => {
                    let columns: Vec<String> =
                        line.split(&separator).map(|s| ["/", s].join("")).collect();

                    if hint_cols == Some(columns.len()) || hint_cols.is_none() {
                        state = State::CheckData;
                        success = true;
                        for c in columns {
                            if let Ok((_, producer)) = signals.add_signal(&c) {
                                producers.push(producer);
                            } else {
                                // Columns are not valid, continue looking for a valid header
                                success = false;
                                break;
                            }
                        }
                    }
                }
                State::CheckData => {
                    if let Ok(values) = line
                        .split(&separator)
                        .map(|c| {
                            c.parse::<f64>()
                                .map_err(|_| anyhow!("Error parsing string: '{c}'"))
                        })
                        .collect::<Result<Vec<_>>>()
                    {
                        if values.len() == producers.len() {
                            // Found a line of data matching the header, this is probably the start of the CSV data!
                            // Send this first data line
                            first_data = Some(buf.clone());

                            break 'line_loop;
                        } else {
                            success = false;
                        }
                    }
                }
            }

            if !success {
                state = State::FindHeader;
                signals = PlotSignals::default();
                producers.clear();
                producers.shrink_to_fit();

                // Echo the line we just read
                println!("{}", line);
            }

            buf.clear();
        }

        (signals, producers, first_data.unwrap())
    }
}

enum TimeIndex {
    Column(usize, Option<f64>),
    Counter(usize),
    Generated(Instant),
}

struct LineSender {
    time_index: TimeIndex,
    senders: Vec<PlotSampleSender>,
    sep: String,
}

impl LineSender {
    fn new(time_index: TimeIndex, senders: Vec<PlotSampleSender>, sep: String) -> Self {
        Self {
            time_index,
            senders,
            sep,
        }
    }

    fn send_line(&mut self, line: &str) -> Result<()> {
        let line = line.trim();
        let data = line
            .split(&self.sep)
            .map(|v| v.parse::<f64>().map_err(anyhow::Error::from))
            .collect::<Result<Vec<f64>>>()?;

        match &mut self.time_index {
            TimeIndex::Column(index, last_time) => {
                if *index < data.len() {
                    let time = *data.get(*index).unwrap();
                    if let Some(last_time) = last_time {
                        if time <= *last_time {
                            return Err(anyhow!("Provided time is not strictly monotonic"));
                        }
                    }
                    *last_time = Some(time);

                    data.into_iter()
                        .zip(self.senders.iter_mut())
                        .enumerate()
                        .filter(|(i, _)| *i != *index)
                        .for_each(|(_, (value, sender))| {
                            let _ = sender.send(PlotSignalSample { time, value });
                        });
                    Ok(())
                } else {
                    Err(anyhow!("Time column index is outside of bounds!"))
                }
            }
            TimeIndex::Generated(start_time) => {
                let time = (Instant::now() - *start_time).as_secs_f64();
                data.into_iter()
                    .zip(self.senders.iter_mut())
                    .enumerate()
                    .for_each(|(_, (value, sender))| {
                        let _ = sender.send(PlotSignalSample { time, value });
                    });
                Ok(())
            }
            TimeIndex::Counter(count) => {
                let time = *count as f64;
                data.into_iter()
                    .zip(self.senders.iter_mut())
                    .enumerate()
                    .for_each(|(_, (value, sender))| {
                        let _ = sender.send(PlotSignalSample { time, value });
                    });

                *count += 1;
                Ok(())
            }
        }
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

    let input: Box<dyn BufRead + Send + 'static> = if let Some(file) = cli.file {
        Box::new(BufReader::new(File::open(file)?))
    } else {
        let stdin = io::stdin();
        Box::new(BufReader::new(stdin))
    };

    let (mut csvplotter, signals) = CSVPlotter::with_signals(
        input,
        cli.columns,
        cli.separator,
        cli.time_index,
        cli.real_time,
        cli.col_hint,
    );

    spawn(move || {
        csvplotter.plot_lines();
    });

    DataInspector::run_native("Rust Data Inspector", signals).unwrap();
    println!("App terminated");
    Ok(())
}
