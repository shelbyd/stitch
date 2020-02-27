#![feature(step_trait)]

pub mod tree_search;
pub use tree_search::*;

mod fast_deque;

pub struct Stitches<P: Problem> {
    mutable: Mutable<P::Space, P::Out>,
    problem: P,
}

impl<P: Problem> Stitches<P> {
    pub fn new(problem: P) -> Self
    where
        P::Space: Default,
        P::Out: Default,
    {
        Stitches {
            problem,
            mutable: Mutable {
                space: P::Space::default(),
                out: P::Out::default(),
            },
        }
    }

    pub fn results(self) -> impl Iterator<Item = P::Out>
    where
        P: Send + Sync + 'static,
        P::Out: Send + 'static,
        P::Space: Send + 'static,
    {
        use spaces::Space;
        use std::sync::{mpsc, Arc, Mutex};

        let (send, recv) = mpsc::channel();

        let mutable = Arc::new(Mutex::new(self.mutable));
        let problem = Arc::new(self.problem);

        for _ in 0..num_cpus::get() {
            let mutable = mutable.clone();
            let send = send.clone();
            let problem = problem.clone();

            std::thread::spawn(move || loop {
                let (batch, mut out) = {
                    let mut lock = mutable.lock().unwrap();
                    let batch = match lock.space.batch(100) {
                        None => break,
                        Some(i) => i,
                    };
                    (batch, lock.out.clone())
                };
                for candidate in batch {
                    let new_result = match problem.check(candidate, &out) {
                        None => continue,
                        Some(result) => result,
                    };
                    out = new_result.clone();
                    mutable.lock().unwrap().out = new_result.clone();
                    send.send(new_result).unwrap();
                }
            });
        }

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

    fn check(
        &self,
        candidate: <<<Self as Problem>::Space as spaces::Space>::Batch as IntoIterator>::Item,
        latest_out: &<Self as Problem>::Out,
    ) -> Option<Self::Out>;
}

pub mod spaces {
    pub trait Space {
        type Batch: IntoIterator;

        fn batch(&mut self, n: usize) -> Option<Self::Batch>;
    }

    #[derive(Default)]
    pub struct LinearSpace<T> {
        unchecked: T,
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
}
