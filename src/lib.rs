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
                    let batch = match lock.space.pluck(100) {
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
        candidate: <<Self as Problem>::Space as spaces::Space>::Candidate,
        latest_out: &<Self as Problem>::Out,
    ) -> Option<Self::Out>;
}

pub mod spaces {
    pub trait Space {
        type Candidate;

        fn pluck(&mut self, n: usize) -> Option<Box<dyn Iterator<Item = Self::Candidate>>>;
    }

    pub struct LinearSpace<T: Linear> {
        unchecked: T,
    }

    impl<T: Linear> Default for LinearSpace<T> {
        fn default() -> Self {
            Self {
                unchecked: T::start(),
            }
        }
    }

    impl<T: Linear> Space for LinearSpace<T> {
        type Candidate = T;

        fn pluck(&mut self, n: usize) -> Option<Box<dyn Iterator<Item = T>>> {
            let next = self.unchecked.increment(n)?;
            let result = self.unchecked.iter_to(&next);
            self.unchecked = next;
            result
        }
    }

    pub trait Linear: Sized {
        fn start() -> Self;
        fn increment(&self, n: usize) -> Option<Self>;
        fn iter_to(&self, next: &Self) -> Option<Box<dyn Iterator<Item = Self>>>;
    }

    impl Linear for u64 {
        fn start() -> Self {
            0
        }

        fn increment(&self, n: usize) -> Option<Self> {
            self.checked_add(n as u64)
        }

        fn iter_to(&self, next: &Self) -> Option<Box<dyn Iterator<Item = Self>>> {
            Some(Box::new((*self..*next).into_iter()))
        }
    }
}
