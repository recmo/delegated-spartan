use std::{
    fmt::{self, Display, Formatter},
    hint::black_box,
    time::{Duration, Instant},
};

/// Repeats a function for at least 5 seconds and returns the minimum duration.
/// This is appropriate for long running benchmarks. For micro-benchmarks,
/// the `criterion` crate is more accurate.
pub fn time<A, F: FnMut() -> A>(mut f: F) -> f64 {
    const DURATION: Duration = Duration::from_secs(5);
    let mut minimum = f64::NAN;
    let total = Instant::now();
    while total.elapsed() < DURATION {
        let bench = Instant::now();
        black_box(f());
        let duration = bench.elapsed().as_secs_f64();
        minimum = minimum.min(duration);
    }
    minimum
}

pub fn human(value: impl Into<f64>) -> Human {
    Human(value.into())
}

pub struct Human(f64);

impl Display for Human {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        const SI_SMALL: &'static str = "mμnpfazyrq";
        const SI_LARGE: &'static str = "kMGTPEZYRQ";
        let log10 = if self.0 == 0.0 {
            0.0
        } else {
            self.0.abs().log10()
        };
        let si_power = (log10 / 3.0).floor() as isize;
        let digits = f.precision().unwrap_or(3) - 1 - (log10 - 3.0 * si_power as f64) as usize;
        if si_power < 0 {
            let value = self.0 * 10_f64.powi((-si_power * 3) as i32);
            let suffix = SI_SMALL.chars().nth((-si_power - 1) as usize).unwrap();
            write!(f, "{value:.digits$} {suffix}")
        } else if si_power > 0 {
            let value = self.0 * 10_f64.powi((-si_power * 3) as i32);
            let suffix = SI_LARGE.chars().nth((si_power - 1) as usize).unwrap();
            write!(f, "{value:.digits$} {suffix}")
        } else {
            write!(f, "{:.digits$} ", self.0)
        }
    }
}
