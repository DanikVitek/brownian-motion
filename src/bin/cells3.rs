use std::{
    iter::{once, repeat},
    sync::{mpsc, Arc, Barrier, RwLock},
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

    let crystal: Arc<RwLock<Box<[usize]>>> = Arc::new(RwLock::new(
        once(impurities.get())
            .chain(repeat(0))
            .take(cells.get())
            .collect(),
    ));
    let tick_barrier = Arc::new(Barrier::new(impurities.get()));

    let (event_sender, event_receiver) = mpsc::channel::<Event>();

    let log_thread = thread::spawn({
        reclone!(crystal, event_sender);

        move || {
            thread::scope(|s| {
                let (total_transitions_sender, total_transitions_receiver) =
                    mpsc::sync_channel::<u64>(0);
                let event_handler =
                    spawn_scoped_event_handler(s, event_receiver, total_transitions_sender);

                let start = Instant::now();
                event_sender.send(Event::AskForTotalTransitions).unwrap();
                print_step(
                    &crystal.read().unwrap(),
                    start,
                    total_transitions_receiver.recv().unwrap(),
                );
                let mut discrete_step_start = start;
                while start.elapsed() < simulation_duration {
                    if discrete_step_start.elapsed() >= log_step_duration {
                        event_sender.send(Event::AskForTotalTransitions).unwrap();
                        print_step(
                            &crystal.read().unwrap(),
                            start,
                            total_transitions_receiver.recv().unwrap(),
                        );
                        discrete_step_start = Instant::now();
                    }
                }
                event_sender.send(Event::Quit).unwrap();

                print_step(
                    &crystal.read().unwrap(),
                    start,
                    event_handler.join().unwrap(),
                );
            });
        }
    });
    for _ in 0..impurities.get() {
        reclone!(tick_barrier, crystal, event_sender);
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut i: usize = 0;
            loop {
                tick_barrier.wait();

                let dir = if rng.gen::<f64>() > transition_probability {
                    Direction::Right
                } else {
                    Direction::Left
                };
                if i == 0 && dir.is_left() || i == cells.get() - 1 && dir.is_right() {
                    if event_sender.send(Event::ParticleMoved).is_err() {
                        break;
                    }
                    continue;
                }

                let next = dir.next(i, cells).unwrap();

                {
                    let mut crystal = crystal.write().unwrap();
                    crystal[i] -= 1;
                    crystal[next] += 1;
                }
                if event_sender.send(Event::ParticleMoved).is_err() {
                    break;
                }

                i = next;
            }
        });
    }

    log_thread.join().unwrap();
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
