pub mod tree_search;
pub use tree_search::*;

mod fast_deque;

pub struct Stitches<S: spaces::Space, State, C> {
    mutable: Mutable<S, State>,
    checker: C,
}

impl<S: spaces::Space, State, C> Stitches<S, State, C>
where
    C: Checker<S::Candidate, State>,
{
    pub fn new(space: S, checker: C) -> Self
    where
        State: Default,
    {
        Stitches {
            mutable: Mutable {
                state: State::default(),
                space,
            },
            checker,
        }
    }

    pub fn results(mut self) -> impl Iterator<Item = State>
    where
        State: Send + Clone + 'static,
        S: 'static,
        C: Send + Sync + 'static,
    {
        use std::sync::{mpsc, Arc, Mutex};

        let (send, recv) = mpsc::channel();

        let mutable = Arc::new(Mutex::new(self.mutable));
        let checker = Arc::new(self.checker);

        for _ in 0..num_cpus::get() {
            let mutable = mutable.clone();
            let send = send.clone();
            let checker = checker.clone();

            std::thread::spawn(move || loop {
                let (batch, mut state) = {
                    let mut lock = mutable.lock().unwrap();
                    let batch = match lock.space.pluck(100) {
                        None => break,
                        Some(i) => i,
                    };
                    (batch, lock.state.clone())
                };
                for candidate in batch {
                    let new_result = match checker.check(candidate, &state) {
                        None => continue,
                        Some(result) => result,
                    };
                    state = new_result.clone();
                    mutable.lock().unwrap().state = new_result.clone();
                    send.send(new_result).unwrap();
                }
            });
        }

        recv.into_iter()
    }
}

struct Mutable<S: spaces::Space, State> {
    state: State,
    space: S,
}

pub trait Checker<C, S> {
    fn check(&self, candidate: C, state: &S) -> Option<S>;
}

impl<F, C, S> Checker<C, S> for F where F: Fn(C, &S) -> Option<S> {
    fn check(&self, candidate: C, state: &S) -> Option<S> {
        self(candidate, state)
    }
}

pub mod spaces {
    pub trait Space: Send {
        type Candidate;

        fn pluck(&mut self, n: usize) -> Option<Box<dyn Iterator<Item = Self::Candidate>>>;
    }

    pub struct LinearSpace<T: Linear + Send> {
        unchecked: T,
    }

    impl<T: Linear + Send> LinearSpace<T> {
        pub fn new() -> Self {
            Self {
                unchecked: T::start(),
            }
        }
    }

    impl<T: Linear + Send> Space for LinearSpace<T> {
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

    enum Range<T: Linear> {
        Bounded(T, T),
        StartingAt(T),
    }
}
