pub mod tree_search;
pub use tree_search::*;

mod fast_deque;

pub struct Stitches<S: spaces::Space, State, C> {
    state: State,
    space: S,
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
            state: State::default(),
            space,
            checker,
        }
    }

    pub fn results(mut self) -> impl Iterator<Item = State>
    where
        State: Send + Clone + 'static,
        S: 'static,
        C: Send + 'static,
    {
        use std::sync::mpsc;
        let (send, recv) = mpsc::channel();

        std::thread::spawn(move || loop {
            let iter = match self.space.pluck(100) {
                None => break,
                Some(i) => i,
            };
            for candidate in iter {
                let new_result = match self.checker.check(candidate, &self.state) {
                    None => continue,
                    Some(result) => result,
                };
                self.state = new_result.clone();
                send.send(new_result).unwrap();
            }
        });

        recv.into_iter()
    }
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
