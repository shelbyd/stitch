use std::error::Error;

use stitches::spaces::LinearSpace;
use stitches::{Problem, StdoutReporter, Stitches};

struct MultiplicativePersistence;

impl Problem for MultiplicativePersistence {
    // Uncomment for limiting time for profiling.
    // type Space = stitches::spaces::TimeLimited<LinearSpace<usize>>;
    type Space = LinearSpace<usize>;
    type Out = ResultsState;

    fn initial_space(&mut self) -> Self::Space {
        // Uncomment for limiting time for profiling.
        // stitches::spaces::TimeLimited::new(std::time::Duration::from_secs(10), LinearSpace::default())
        LinearSpace::default()
    }

    fn check(&self, number: usize, results_state: &ResultsState) -> Option<ResultsState> {
        let p = persistence(number);
        if p > results_state.best_persistence {
            Some(ResultsState {
                best_persistence: p,
                best_number: number,
            })
        } else if p == results_state.best_persistence && number < results_state.best_number {
            Some(ResultsState {
                best_persistence: p,
                best_number: number,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone)]
struct ResultsState {
    best_persistence: u8,
    best_number: usize,
}

fn persistence(n: usize) -> u8 {
    if n < 10 {
        return 0;
    }

    let digits = n.to_string();
    if digits.chars().any(|c| c == '0') {
        return 1;
    }

    let product: usize = digits
        .chars()
        .map(|d| d as u32 - '0' as u32)
        .fold(1, |acc, d| acc * (d as usize));
    1 + persistence(product)
}

fn main() -> Result<(), Box<dyn Error>> {
    let stitches = Stitches::new(MultiplicativePersistence, StdoutReporter);

    for result in stitches.results() {
        dbg!(result);
    }

    Ok(())
}
