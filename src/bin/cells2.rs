use std::{
    iter::{once, repeat, repeat_with},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use brownian_motion::{Args, Direction};
use rand::Rng;

fn main() {
    let Args {
        cells,
        impurities,
        transition_probability,
    } = Args::parse();

    thread::scope(|s| {
        let crystal = Arc::new(Mutex::new(
            once(impurities.get())
                .chain(repeat(0))
                .take(cells.get())
                .collect::<Vec<_>>(),
        ));
        let (notify_senders, notify_receivers): (Vec<_>, Vec<_>) =
            repeat_with(|| crossbeam::channel::bounded::<()>(0))
                .take(impurities.get())
                .unzip();
        s.spawn({
            let crystal = crystal.clone();
            move || {
                let start = Instant::now();
                print_step(&crystal, start);
                let mut discrete_step_start = start;
                while start.elapsed() < Duration::from_secs(60) {
                    notify_senders.iter().for_each(|s| s.send(()).unwrap());
                    if discrete_step_start.elapsed() > Duration::from_secs(5) {
                        print_step(&crystal, start);
                        discrete_step_start = Instant::now();
                    }
                }
                print_step(&crystal, start);
            }
        });
        for notifications in notify_receivers {
            let crystal = crystal.clone();
            s.spawn(move || {
                let mut rng = rand::thread_rng();
                let mut i: usize = 0;
                while let Ok(_) = notifications.recv() {
                    let dir = if rng.gen::<f64>() > transition_probability {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                    if i == 0 && dir.is_left() || i == cells.get() - 1 && dir.is_right() {
                        continue;
                    }

                    let next = dir.next(i, cells).unwrap();

                    {
                        let mut crystal = crystal.lock().unwrap();
                        crystal[i] -= 1;
                        crystal[next] += 1;
                    }

                    i = next;
                }
            });
        }
    });
}

fn print_step(crystal: &Mutex<Vec<usize>>, start: Instant) {
    let crystal = crystal.lock().unwrap();
    println!(
        "[{}]\t{:?}: {}",
        start.elapsed().as_secs(),
        crystal,
        crystal.iter().sum::<usize>()
    );
}
