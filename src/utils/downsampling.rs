use std::cmp::max;

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
