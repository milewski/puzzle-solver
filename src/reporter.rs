use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use human_format::Formatter;

#[derive(Clone, Copy)]
pub struct Reporter {
    rate: u64,
    interval: Duration,
    report_at: Instant,
}

impl Reporter {
    pub fn clean() -> Self {
        Self {
            rate: 0,
            report_at: Instant::now(),
            interval: Duration::from_secs(60),
        }
    }

    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            rate: 0,
            report_at: Instant::now(),
            interval: Duration::from_secs(60),
        }))
    }

    pub fn update(&mut self, hashes: u64) {
        self.rate += hashes;

        let duration = Instant::now().duration_since(self.report_at);

        if duration.gt(&self.interval) {
            let count = self.rate.div_ceil(duration.as_secs());

            let mut scales = human_format::Scales::new();

            scales.with_base(1000);
            scales.with_suffixes(vec!["H/s", "KH/s", "MH/s", "GH/s", "TH/s", "PH/s", "EH/s"]);

            let number = Formatter::new()
                .with_scales(scales)
                .format(count as f64);

            println!("Hash rate: {}", number);
            self.rate = 0;
            self.report_at = Instant::now();
        }
    }
}