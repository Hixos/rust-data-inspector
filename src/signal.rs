pub struct SignalSample {
    pub t: f64,
    pub y: f64,
}

pub struct Signal {
    name: String,
    time: Vec<f64>,
    data: Vec<f64>,
}

impl Signal {
    pub fn new(name: &str) -> Signal {
        return Signal {
            name: name.to_string(),
            time: vec![],
            data: vec![],
        };
    }

    pub fn push(&mut self, v: SignalSample) {
        self.time.push(v.t);
        self.data.push(v.y);
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn time(&self) -> &[f64] {
        &self.time
    }

    pub fn data(&self) -> &[f64] {
        &self.data
    }

    pub fn get_last_sample(&self) -> Option<SignalSample> {
        self.time.last().and_then(|t| {
            Some(SignalSample {
                t: *t,
                y: *self.data.last().unwrap(),
            })
        })
    }
}
