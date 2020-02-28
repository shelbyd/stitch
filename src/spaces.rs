use std::time::{Duration, Instant};

pub trait Space {
    type Batch: ExactSizeIterator;

    fn batch(&mut self, n: usize) -> Option<Self::Batch>;
}

#[derive(Debug)]
pub struct LinearSpace<T> {
    unchecked: Option<T>,
}

impl<T> LinearSpace<T> {
    pub fn new(start: T) -> Self {
        LinearSpace {
            unchecked: Some(start),
        }
    }
}

impl<T> Default for LinearSpace<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Space for LinearSpace<T>
where
    T: Clone + std::iter::Step,
{
    type Batch = MyRangeInclusive<T>;

    fn batch(&mut self, n: usize) -> Option<Self::Batch> {
        let start = self.unchecked.as_ref()?;
        let next = match start.add_usize(n - 1) {
            Some(m) => m,
            None => {
                let mut result = start.clone();
                let mut k = n / 2;
                while k > 0 {
                    if let Some(m) = result.add_usize(k) {
                        result = m;
                    }
                    k = k / 2;
                }
                result
            }
        };

        let result = MyRangeInclusive::new(start.clone(), next.clone());
        self.unchecked = next.add_usize(1);
        Some(result)
    }
}

pub struct MyRangeInclusive<T> {
    start: T,
    end: T,
    done: bool,
}

impl<T> MyRangeInclusive<T> {
    fn new(start: T, end: T) -> Self {
        MyRangeInclusive {
            start,
            end,
            done: false,
        }
    }
}

impl<T: Clone + std::iter::Step> std::iter::Iterator for MyRangeInclusive<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if self.start == self.end {
            self.done = true;
            return Some(self.end.clone());
        }

        let n = self.start.add_one();
        Some(std::mem::replace(&mut self.start, n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = T::steps_between(&self.start, &self.end)
            .expect("Got more than usize steps between start and end")
            + 1;
        (size, Some(size))
    }
}

impl<T: Clone + std::iter::Step> std::iter::ExactSizeIterator for MyRangeInclusive<T> {}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod linaer_space {
        use super::*;

        #[test]
        fn returns_all_values() {
            let end = std::usize::MAX - 4;
            let mut space = LinearSpace::<usize>::default();

            dbg!(&space);
            let batch = space.batch(end).unwrap();
            assert_eq!(batch.len(), end);

            let batch: Vec<usize> = space
                .batch(5)
                .unwrap()
                .map(|n| std::usize::MAX - n)
                .collect();
            assert_eq!(batch, [4, 3, 2, 1, 0]);
        }

        #[test]
        fn returns_remaining_values_when_asked_for_batch_beyond_remaining_size() {
            let end = std::usize::MAX - 4;
            let mut space = LinearSpace::<usize>::default();

            let batch = space.batch(end);

            let batch: Vec<usize> = space
                .batch(6)
                .unwrap()
                .map(|n| std::usize::MAX - n)
                .collect();
            assert_eq!(batch, [4, 3, 2, 1, 0]);
        }
    }
}
