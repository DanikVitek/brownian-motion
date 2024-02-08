use std::{num::NonZeroUsize, ops::RangeInclusive, thread};

use clap::{CommandFactory, Parser};

const PROBABILITY_RANGE: RangeInclusive<f64> = 0.0..=1.0;

fn main() {
    let Args {
        cells,
        impurities,
        transition_probability,
    } = Args::parse();
    if !PROBABILITY_RANGE.contains(&transition_probability) {
        Args::command()
            .error(clap::error::ErrorKind::InvalidValue, "p must be in [0; 1]")
            .exit();
    }

    thread::scope(|s| {

    });
}

#[derive(Parser)]
#[command(author, about, long_about = None)]
struct Args {
    /// Number of cells in the "crystal"
    #[arg(short = 'N', long, default_value_t = NonZeroUsize::new(10).unwrap())]
    cells: NonZeroUsize,
    /// Number of impurities in the "crystal"
    #[arg(short = 'K', long, default_value_t = NonZeroUsize::new(3).unwrap())]
    impurities: NonZeroUsize,
    /// Probability of a particle transitioning to the next cell at any given time step
    #[arg(short = 'p', long, default_value_t = 0.5)]
    transition_probability: f64,
}
