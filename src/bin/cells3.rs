use std::{
    iter::{once, repeat, repeat_with},
    sync::{mpsc, Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

use brownian_motion::{spawn_scoped_event_handler, Args, Direction, Event};
use rand::Rng;

fn main() {
    let Args {
        cells,
        impurities,
        transition_probability,
    } = Args::parse();

    thread::scope(|s| {
        let crystal: Arc<RwLock<Box<[usize]>>> = Arc::new(RwLock::new(
            once(impurities.get())
                .chain(repeat(0))
                .take(cells.get())
                .collect(),
        ));
        let (notify_senders, notify_receivers): (Vec<_>, Vec<_>) =
            repeat_with(|| crossbeam::channel::bounded::<()>(0))
                .take(impurities.get())
                .unzip();

        let (event_sender, event_receiver) = mpsc::channel::<Event>();
        let (total_transitions_sender, total_transitions_receiver) = mpsc::channel::<u64>();

        let event_handler = spawn_scoped_event_handler(s, event_receiver, total_transitions_sender);

        s.spawn({
            let crystal = crystal.clone();
            let event_sender = event_sender.clone();
            move || {
                let start = Instant::now();
                event_sender.send(Event::AskForTotalTransitions).unwrap();
                print_step(
                    &crystal.read().unwrap(),
                    start,
                    total_transitions_receiver.recv().unwrap(),
                );
                let mut discrete_step_start = start;
                while start.elapsed() < Duration::from_secs(60) {
                    notify_senders.iter().for_each(|s| s.send(()).unwrap());
                    if discrete_step_start.elapsed() > Duration::from_secs(5) {
                        event_sender.send(Event::AskForTotalTransitions).unwrap();
                        print_step(
                            &crystal.read().unwrap(),
                            start,
                            total_transitions_receiver.recv().unwrap(),
                        );
                        discrete_step_start = Instant::now();
                    }
                }
                drop(notify_senders);
                drop(event_sender);
                print_step(
                    &crystal.read().unwrap(),
                    start,
                    event_handler.join().unwrap(),
                );
            }
        });
        for notifications in notify_receivers {
            let crystal = crystal.clone();
            let event_sender = event_sender.clone();
            s.spawn(move || {
                let mut rng = rand::thread_rng();
                let mut i: usize = 0;
                while notifications.recv().is_ok() {
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
                        let mut crystal = crystal.write().unwrap();
                        crystal[i] -= 1;
                        crystal[next] += 1;
                    }
                    _ = event_sender.send(Event::ParticleMoved);

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
