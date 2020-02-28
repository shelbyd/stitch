#![feature(step_trait)]

use std::cell::Cell;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub mod reporter;
pub use reporter::*;

pub mod spaces;
pub use spaces::*;

pub mod tree_search;
pub use tree_search::*;

mod fast_deque;

pub struct Stitches<P: Problem, R: Reporter<P>> {
    mutable: Mutable<P::Space, P::Out>,
    problem: P,
    reporter: R,
}

impl<P: Problem, R: Reporter<P>> Stitches<P, R> {
    pub fn new(mut problem: P, reporter: R) -> Self
    where
        P::Out: Default,
    {
        Stitches {
            mutable: Mutable {
                space: problem.initial_space(),
                out: P::Out::default(),
                stats: Stats::default(),
            },
            problem,
            reporter,
        }
    }

    pub fn results(self) -> impl Iterator<Item = P::Out>
    where
        P: Send + Sync + 'static,
        P::Out: Send + 'static,
        P::Space: Send + 'static,
        R: Send + 'static,
    {
        use spaces::Space;
        use std::sync::{mpsc, Arc};

        let (send, recv) = mpsc::channel();

        let mutable = Arc::new(Mutex::new(self.mutable));
        let problem = Arc::new(self.problem);
        let reporter = self.reporter;

        for _ in 0..num_cpus::get() {
            let mutable = mutable.clone();
            let send = send.clone();
            let problem = problem.clone();
            let mut batch_size_optimizer = BatchSizeOptimizer::new(Duration::from_millis(10));

            std::thread::spawn(move || loop {
                let (batch, out) = {
                    let mut lock = mutable.lock().unwrap();
                    let batch = match lock.space.batch(batch_size_optimizer.next_batch()) {
                        None => break,
                        Some(i) => i,
                    };
                    lock.stats.count += batch.len();
                    (batch, lock.out.clone())
                };

                let new_result = batch.fold(out.clone(), |last, candidate| {
                    problem.check(candidate, &last).unwrap_or(last)
                });
                if new_result != out {
                    mutable.lock().unwrap().out = new_result.clone();
                    send.send(new_result).unwrap();
                }
            });
        }

        std::thread::spawn(move || {
            let mut reporter = reporter;
            let mutable = mutable.clone();
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let mut locked = mutable.lock().unwrap();
                reporter.report_on(&locked.space, &locked.out, &locked.stats);
                locked.stats.clear();
            }
        });

        recv.into_iter()
    }
}

struct Mutable<S: spaces::Space, O> {
    out: O,
    space: S,
    stats: Stats,
}

pub trait Problem {
    type Space: spaces::Space;
    type Out: Clone + Eq;

    fn initial_space(&mut self) -> Self::Space;

    fn check(
        &self,
        candidate: <<<Self as Problem>::Space as spaces::Space>::Batch as IntoIterator>::Item,
        latest_out: &<Self as Problem>::Out,
    ) -> Option<Self::Out>;
}

#[derive(Debug)]
pub struct Stats {
    recording_since: Mutex<Cell<Instant>>,
    count: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            recording_since: Mutex::new(Cell::new(Instant::now())),
            count: 0,
        }
    }
}

impl Stats {
    fn clear(&mut self) {
        self.count = 0;
        self.recording_since.lock().unwrap().set(Instant::now());
    }

    pub fn throughput(&self) -> f64 {
        let duration_since_read = self.recording_since.lock().unwrap().get().elapsed();
        self.count as f64 / duration_since_read.as_secs_f64()
    }
}

struct BatchSizeOptimizer {
    target: Duration,
    last_batch: Option<(Instant, usize)>,
}

impl BatchSizeOptimizer {
    fn new(target: Duration) -> Self {
        BatchSizeOptimizer {
            target,
            last_batch: None,
        }
    }

    fn next_batch(&mut self) -> usize {
        let batch_size = match self.last_batch {
            None => 100,
            Some((at, size)) => match at.elapsed().cmp(&self.target) {
                std::cmp::Ordering::Less => size * 101 / 100,
                std::cmp::Ordering::Equal => size,
                std::cmp::Ordering::Greater => size * 99 / 100,
            },
        };
        let batch_size = std::cmp::max(batch_size, 1);
        self.last_batch = Some((Instant::now(), batch_size));
        batch_size
    }
}
