use std::time::{Duration, Instant};

pub trait Space {
    type Batch: ExactSizeIterator;

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
    core::ops::Range<T>: ExactSizeIterator,
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
