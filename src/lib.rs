#![feature(step_trait)]

use metered::{clear::Clear, measure, HitCount};
use std::time::{Duration, Instant};

pub mod reporter;
pub use reporter::*;

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
        use std::sync::{mpsc, Arc, Mutex};

        let (send, recv) = mpsc::channel();

        let mutable = Arc::new(Mutex::new(self.mutable));
        let problem = Arc::new(self.problem);
        let reporter = self.reporter;

        let stats = Arc::new(Stats::default());

        for _ in 0..num_cpus::get() {
            let mutable = mutable.clone();
            let send = send.clone();
            let problem = problem.clone();
            let stats = stats.clone();
            let mut batch_size_optimizer = BatchSizeOptimizer::new(Duration::from_millis(10));

            std::thread::spawn(move || loop {
                let (batch, mut out) = {
                    let mut lock = mutable.lock().unwrap();
                    let batch = match lock.space.batch(batch_size_optimizer.next_batch()) {
                        None => break,
                        Some(i) => i,
                    };
                    (batch, lock.out.clone())
                };
                for candidate in batch {
                    let count = &stats.count;
                    let result = measure!(count, { problem.check(candidate, &out) });
                    let new_result = match result {
                        None => continue,
                        Some(result) => result,
                    };
                    out = new_result.clone();
                    mutable.lock().unwrap().out = new_result.clone();
                    send.send(new_result).unwrap();
                }
            });
        }

        std::thread::spawn(move || {
            let stats = stats.clone();
            let mut reporter = reporter;
            let mutable = mutable.clone();
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let locked = mutable.lock().unwrap();
                reporter.report_on(&locked.space, &locked.out, &stats);
                stats.clear();
            }
        });

        recv.into_iter()
    }
}

struct Mutable<S: spaces::Space, O> {
    out: O,
    space: S,
}

pub trait Problem {
    type Space: spaces::Space;
    type Out: Clone;

    fn initial_space(&mut self) -> Self::Space;

    fn check(
        &self,
        candidate: <<<Self as Problem>::Space as spaces::Space>::Batch as IntoIterator>::Item,
        latest_out: &<Self as Problem>::Out,
    ) -> Option<Self::Out>;
}

#[derive(Debug)]
pub struct Stats {
    recording_since: Instant,
    count: HitCount,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            recording_since: Instant::now(),
            count: HitCount::default(),
        }
    }
}

impl Stats {
    fn clear(&self) {
        self.count.clear();
    }

    pub fn throughput(&self) -> f64 {
        self.count.0.get() as f64 / self.recording_since.elapsed().as_secs_f64()
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

pub mod spaces {
    use std::time::{Duration, Instant};

    pub trait Space {
        type Batch: IntoIterator;

        fn batch(&mut self, n: usize) -> Option<Self::Batch>;
    }

    #[derive(Debug, Default)]
    pub struct LinearSpace<T> {
        unchecked: T,
    }

    impl<T> LinearSpace<T> {
        pub fn new(start: T) -> Self {
            LinearSpace { unchecked: start }
        }
    }

    impl<T> Space for LinearSpace<T>
    where
        T: Clone + std::iter::Step,
    {
        type Batch = core::ops::Range<T>;

        fn batch(&mut self, n: usize) -> Option<Self::Batch> {
            let next = self.unchecked.add_usize(n)?;
            let result = self.unchecked.clone()..next.clone();
            self.unchecked = next;
            Some(result)
        }
    }

    #[derive(Debug)]
    pub struct TimeLimited<S: Space> {
        space: S,
        first_batch: Option<Instant>,
        limit: Duration,
    }

    impl<S: Space> TimeLimited<S> {
        pub fn new(limit: Duration, space: S) -> Self {
            TimeLimited {
                limit,
                space,
                first_batch: None,
            }
        }
    }

    impl<S: Space> Space for TimeLimited<S> {
        type Batch = S::Batch;

        fn batch(&mut self, n: usize) -> Option<Self::Batch> {
            if let None = self.first_batch {
                self.first_batch = Some(Instant::now());
            }
            if self.first_batch.unwrap().elapsed() > self.limit {
                return None;
            }
            self.space.batch(n)
        }
    }
}
