use crate::fast_deque::*;
use std::sync::Mutex;

pub trait TreeSpace: Sync {
    type Candidate: Send;

    fn initial(&self) -> Box<Iterator<Item = Self::Candidate>>;

    fn following<'s>(&'s self, candidate: Self::Candidate) -> Box<Iterator<Item = Self::Candidate> + 's>;

    fn each(&self, candidate: &Self::Candidate);
}

pub fn tree_search<S: TreeSpace>(space: S) {
    let threads = rayon::current_num_threads();
    let mut undone = FastDeque::new(threads);
    undone.extend(space.initial());
    let undone = Mutex::new(undone);
    rayon::scope(|s| {
        for _ in 0..threads {
            s.spawn(|_| {
               work_loop(&space, &undone);
            });
        }
    });
}

fn work_loop<S: TreeSpace>(space: &S, work_queue: &Mutex<FastDeque<S::Candidate>>) {
    let amount = 1024;
    loop {
        let work = work_queue.lock().unwrap().split_off(amount);
        let new_items = work.into_iter()
            .flat_map(|v| v)
            .inspect(|c| space.each(c))
            .flat_map(|c| space.following(c))
            .collect::<Vec<_>>();
        work_queue.lock().unwrap().give(new_items);
    }
}
