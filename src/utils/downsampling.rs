use std::{
    cmp::max,
    ops::{Range, RangeInclusive},
};

use rust_data_inspector_signals::PlotSignal;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DownsamplingMethod {
    Lttb,
    Decimation,
}

pub fn decimate(time: &[f64], max_num_out: usize) -> Vec<usize> {
    let step = max(time.len() / max_num_out, 1);
    time.iter()
        .enumerate()
        .step_by(step)
        .map(|(i, _)| i)
        .collect()
}

pub fn find_visible_index_range(
    signal: &PlotSignal,
    visible_range: &RangeInclusive<f64>,
    last_index_range: Option<Range<usize>>,
) -> Option<Range<usize>> {
    if signal.time().is_empty() {
        return None;
    }

    let mut range_i = last_index_range.unwrap_or(0..1);

    let left_t = *signal.time().get(range_i.start).unwrap();
    let right_t = *signal.time().get(range_i.end - 1).unwrap();

    let left_bound = *visible_range.start();
    let right_bound = *visible_range.end();

    if left_t < left_bound {
        for (i, &t) in signal.time().iter().enumerate().skip(range_i.start) {
            if t < left_bound {
                range_i.start = i;
            } else {
                break;
            }
        }
    } else {
        for (i, &t) in signal
            .time()
            .iter()
            .enumerate()
            .rev()
            .skip(signal.time().len() - range_i.start)
        {
            range_i.start = i;
            if t < left_bound {
                break;
            }
        }
    }

    if right_t < right_bound {
        for (i, &t) in signal.time().iter().enumerate().skip(range_i.end) {
            range_i.end = i + 1;
            if t > right_bound {
                break;
            }
        }
    } else {
        for (i, &t) in signal
            .time()
            .iter()
            .enumerate()
            .rev()
            .skip(signal.time().len() - range_i.end)
        {
            if t > right_bound {
                range_i.end = i + 1;
            } else {
                break;
            }
        }
    }

    Some(range_i.start..range_i.end)
}
