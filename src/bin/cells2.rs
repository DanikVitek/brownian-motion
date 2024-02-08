use std::{
    iter::{once, repeat, repeat_with},
    sync::{
        atomic::{self, AtomicU64},
        Arc, Mutex,
    },
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
                .collect::<Box<[_]>>(),
        ));
        let (notify_senders, notify_receivers): (Vec<_>, Vec<_>) =
            repeat_with(|| crossbeam::channel::bounded::<()>(0))
                .take(impurities.get())
                .unzip();
        let total_transitions = Arc::new(AtomicU64::new(0));
        s.spawn({
            let crystal = crystal.clone();
            let total_transitions = total_transitions.clone();
            move || {
                let start = Instant::now();
                print_step(
                    &crystal.lock().unwrap(),
                    start,
                    total_transitions.load(atomic::Ordering::Relaxed),
                );
                let mut discrete_step_start = start;
                while start.elapsed() < Duration::from_secs(60) {
                    notify_senders.iter().for_each(|s| s.send(()).unwrap());
                    if discrete_step_start.elapsed() > Duration::from_secs(5) {
                        print_step(
                            &crystal.lock().unwrap(),
                            start,
                            total_transitions.load(atomic::Ordering::Relaxed),
                        );
                        discrete_step_start = Instant::now();
                    }
                }
                print_step(
                    &crystal.lock().unwrap(),
                    start,
                    total_transitions.load(atomic::Ordering::Relaxed),
                );
            }
        });
        for notifications in notify_receivers {
            let crystal = crystal.clone();
            let total_transitions = total_transitions.clone();
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
                    total_transitions.fetch_add(1, atomic::Ordering::SeqCst);

                    i = next;
                }
            });
        }
    });
}

fn print_step(crystal: &[usize], start: Instant, total_transitions: u64) {
    println!(
        "[{}]\t{:?}. Particles: {}, Transitions: {}",
        start.elapsed().as_secs(),
        crystal,
        crystal.iter().sum::<usize>(),
        total_transitions,
    );
}
