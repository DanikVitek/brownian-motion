use std::{num::NonZeroUsize, ops::RangeInclusive};

use clap::{CommandFactory, Parser};

const PROBABILITY_RANGE: RangeInclusive<f64> = 0.0..=1.0;

#[derive(Parser)]
#[command(author, about, long_about = None)]
pub struct Args {
    /// Number of cells in the "crystal"
    #[arg(short = 'N', long, default_value_t = NonZeroUsize::new(10).unwrap())]
    pub cells: NonZeroUsize,
    /// Number of impurities in the "crystal"
    #[arg(short = 'K', long, default_value_t = NonZeroUsize::new(3).unwrap())]
    pub impurities: NonZeroUsize,
    /// Probability of a particle transitioning to the next cell at any given time step
    #[arg(short = 'p', long, default_value_t = 0.5)]
    pub transition_probability: f64,
}

impl Args {
    pub fn parse() -> Self {
        let args = <Args as Parser>::parse();
        if !PROBABILITY_RANGE.contains(&args.transition_probability) {
            Args::command()
                .error(clap::error::ErrorKind::InvalidValue, "p must be in [0; 1]")
                .exit();
        }
        args
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    /// Returns `true` if the direction is [`Left`].
    ///
    /// [`Left`]: Direction::Left
    #[must_use]
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }

    /// Returns `true` if the direction is [`Right`].
    ///
    /// [`Right`]: Direction::Right
    #[must_use]
    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right)
    }

    pub fn next(self, i: usize, max: NonZeroUsize) -> Option<usize> {
        match self {
            Self::Left => i.checked_sub(1),
            Self::Right => i.checked_add(1).filter(|&i| i < max.get()),
        }
    }
}
