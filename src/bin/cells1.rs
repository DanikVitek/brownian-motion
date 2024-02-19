use std::{
    iter::{once, repeat_with},
    sync::{
        atomic::{self, AtomicBool, AtomicUsize},
        mpsc, Barrier,
    },
    thread,
    time::Instant,
};

use brownian_motion::{reclone, spawn_scoped_event_handler, Args, Direction, Event};
use rand::Rng;

fn main() {
    let (
        Args {
            cells,
            impurities,
            transition_probability,
            ..
        },
        log_step_duration,
        simulation_duration,
    ) = {
        let args = Args::parse();
        let log_step_duration = args.log_step_duration();
        let simulation_duration = args.simulation_duration();
        (args, log_step_duration, simulation_duration)
    };

    let quit = &AtomicBool::new(false);
    let tick_barrier = &Barrier::new(impurities.get());
    let crystal: &[AtomicUsize] = &once(AtomicUsize::new(impurities.get()))
        .chain(repeat_with(|| AtomicUsize::new(0)))
        .take(cells.get())
        .collect::<Box<[_]>>();

    thread::scope(|s| {
        let (event_sender, event_receiver) = mpsc::channel::<Event>();
        let (total_transitions_sender, total_transitions_receiver) = mpsc::channel::<u64>();

        let event_handler = spawn_scoped_event_handler(s, event_receiver, total_transitions_sender);

        s.spawn({
            reclone!(event_sender);
            move || {
                let start = Instant::now();
                event_sender.send(Event::AskForTotalTransitions).unwrap();
                print_step(&crystal, start, total_transitions_receiver.recv().unwrap());
                let mut discrete_step_start = start;
                while start.elapsed() < simulation_duration {
                    if discrete_step_start.elapsed() >= log_step_duration {
                        event_sender.send(Event::AskForTotalTransitions).unwrap();
                        print_step(&crystal, start, total_transitions_receiver.recv().unwrap());
                        discrete_step_start = Instant::now();
                    }
                }
                quit.store(true, atomic::Ordering::Relaxed);
                drop(event_sender);
                print_step(&crystal, start, event_handler.join().unwrap());
            }
        });

        for _ in 0..impurities.get() {
            reclone!(event_sender);
            s.spawn(move || {
                let mut rng = rand::thread_rng();
                let mut i: usize = 0;
                while !quit.load(atomic::Ordering::Relaxed) {
                    tick_barrier.wait();

                    let dir = if rng.gen::<f64>() > transition_probability {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                    if i == 0 && dir.is_left() || i == cells.get() - 1 && dir.is_right() {
                        continue;
                    }

                    let next = dir.next(i, cells).unwrap();

                    crystal[i].fetch_sub(1, atomic::Ordering::Relaxed);
                    crystal[next].fetch_add(1, atomic::Ordering::Relaxed);

                    if event_sender.send(Event::ParticleMoved).is_err() {
                        break;
                    }

                    i = next;
                }
            });
        }
    });
}

fn print_step(crystal: &[AtomicUsize], start: Instant, total_transitions: u64) {
    println!(
        "[{}]\t{:?}. Particles: {}, Transitions: {}",
        start.elapsed().as_secs(),
        crystal,
        crystal
            .iter()
            .map(|c| c.load(atomic::Ordering::Relaxed))
            .sum::<usize>(),
        total_transitions,
    );
}
