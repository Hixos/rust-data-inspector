use anyhow::{anyhow, bail, Result};
use clap::Parser;
use rust_data_inspector::datainspector::DataInspector;
use rust_data_inspector_signals::{PlotSignalProducer, PlotSignalSample, PlotSignals};
use std::{
    fs::{read, File},
    hint,
    io::{self, BufRead, BufReader, Seek, Stdin},
    path::Path,
    thread::spawn,
    time::Instant,
};

trait BufReadSeek: BufRead + Seek {}

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
}

impl CSVPlotter {
    fn with_signals<R: BufReadSeek + Send + 'static>(
        signals: &mut PlotSignals,
        mut reader: R,
        mut columns: Vec<String>,
        separator: String,
        time_index: Option<usize>,
        real_time: bool,
        hint_cols: Option<usize>,
    ) -> Result<Self> {
        let (signals, senders) = Self::find_data(&mut reader, hint_cols, separator);

        Ok(CSVPlotter {
            buf: Box::new(reader),
            columns,
            senders,
            separator,
            time_index,
            real_time,
        })
    }

    fn find_data<R: BufReadSeek + Send + 'static>(
        reader: &mut R,
        hint_cols: Option<usize>,
        separator: String,
    ) -> (PlotSignals, Vec<PlotSignalProducer>) {
        enum State {
            FindHeader,
            CheckData,
        }

        let mut signals = PlotSignals::default();
        let mut producers: Vec<PlotSignalProducer> = vec![];

        let mut state = State::FindHeader;
        let mut buf = String::new();

        while let Ok(len) = reader.read_line(&mut buf) {
            let mut success = false;
            match state {
                State::FindHeader => {
                    let columns: Vec<String> =
                        buf.split(&separator).map(|s| ["/", s].join("")).collect();

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
                    if let Ok(values) = buf
                        .split(&separator)
                        .map(|c| {
                            c.parse::<f64>()
                                .map_err(|_| anyhow!("Error parsing string: '{c}'"))
                        })
                        .collect::<Result<Vec<_>>>()
                    {
                        if values.len() == producers.len() {
                            // Found a line of data matching the header, this is probably the start of the CSV data!
                            success = true;

                            // Seek back so we can plot the first data line
                            reader.seek(io::SeekFrom::Current(-(len as i64))).unwrap();
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
                println!("{}", buf);
            }

            buf.clear();
        }

        (signals, producers)
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
                .map(|c| {
                    c.parse::<f64>()
                        .map_err(|_| anyhow!("Error parsing string: '{c}'"))
                })
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

impl BufReadSeek for BufReader<File> {

}

impl BufReadSeek for BufReader<Stdin> {

}

fn main() -> Result<()> {
    set_thread_panic_hook();
    let cli = Cli::parse();

    let mut signals = PlotSignals::default();

    let input: Box<dyn BufReadSeek + Send + 'static> = if let Some(file) = cli.file {
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
        cli.col_hint
    )?;

    spawn(move || {
        csvplotter.plot_lines().expect("Error reading csv file");
    });

    DataInspector::run_native("Rust Data Inspector", signals).unwrap();
    println!("App terminated");
    Ok(())
}
