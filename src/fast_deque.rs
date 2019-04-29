use crate::prelude::*;

pub struct FastDeque<T> {
    collection: VecDeque<Vec<T>>,
    suggested_outer_len: usize,
}

impl<T> FastDeque<T> {
    pub fn new(suggested_outer_len: usize) -> FastDeque<T> {
        FastDeque {
            collection: VecDeque::new(),
            suggested_outer_len,
        }
    }

    pub fn split_off(&mut self, mut amount: usize) -> Vec<Vec<T>> {
        let mut result = Vec::new();
        while amount > 0 {
            let mut front = match self.collection.pop() {
                Some(f) => f,
                None => break,
            };
            if amount < front.len() {
                let end = front.split_off(amount);
                self.collection.push_front(end.into());
            }
            amount -= front.len();
            result.push(front);
            if self.collection.len() < self.suggested_outer_len {
                break;
            }
        }
        result
    }

    pub fn outer_len(&self) -> usize {
        self.collection.len()
    }

    pub fn len(&self) -> usize {
        self.collection.iter().map(|v| v.len()).sum()
    }

    pub fn get(&self, mut index: usize) -> Option<&T> {
        for vec in self.collection.iter() {
            if (index + 1) <= vec.len() {
                return vec.get(index);
            } else {
                index -= vec.len();
            }
        }
        None
    }

    pub fn give(&mut self, mut vec: Vec<T>) {
        if self.collection.len() < self.suggested_outer_len {
            self.collection
                .push_back(vec.split_off(vec.len() / 2).into());
        }
        self.collection.push_back(vec.into());
    }

    pub fn peek_head(&self) -> Option<&T> {
        self.collection.peek().and_then(|v| v.first())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.collection.iter().flat_map(|v| v.iter())
    }
}

impl<T> std::iter::Extend<T> for FastDeque<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.give(iter.into_iter().collect());
    }
}

impl<T> ParallelExtend<T> for FastDeque<T>
where
    T: Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        par_iter
            .into_par_iter()
            .fold(
                || Vec::new(),
                |mut vec, item| {
                    vec.push(item);
                    vec
                },
            )
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|vec| self.give(vec))
    }
}
