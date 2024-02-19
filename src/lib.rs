use std::{
    num::NonZeroUsize,
    ops::RangeInclusive,
    sync::mpsc,
    thread::{Scope, ScopedJoinHandle},
    time::Duration,
};

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
    /// Probability of a particle transitioning to the left cell instead of right cell at any given time step
    #[arg(short = 'p', long, default_value_t = 0.5)]
    pub transition_probability: f64,
    /// Duration between logging the state of the "crystal"
    #[arg(short = 's', long, default_value_t = 5.)]
    log_step_duration: f64,
    /// Duration of the simulation
    #[arg(short = 't', long, default_value_t = 60.)]
    simulation_duration: f64,
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

    pub fn log_step_duration(&self) -> Duration {
        Duration::from_secs_f64(self.log_step_duration)
    }

    pub fn simulation_duration(&self) -> Duration {
        Duration::from_secs_f64(self.simulation_duration)
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

pub enum Event {
    ParticleMoved,
    AskForTotalTransitions,
    Quit,
}

pub fn spawn_scoped_event_handler<'scope>(
    scope: &'scope Scope<'scope, '_>,
    event_receiver: mpsc::Receiver<Event>,
    total_transitions_sender: mpsc::SyncSender<u64>,
) -> ScopedJoinHandle<'scope, u64> {
    scope.spawn(move || {
        let mut transitions: u64 = 0;
        while let Ok(event) = event_receiver.recv() {
            match event {
                Event::ParticleMoved => transitions += 1,
                Event::AskForTotalTransitions => {
                    total_transitions_sender.send(transitions).unwrap();
                }
                Event::Quit => break,
            }
        }
        transitions
    })
}

#[macro_export]
macro_rules! reclone {
    ($($v:ident),+ $(,)?) => {
        $(let $v = $v.clone();)+
    };
}
