use std::error::Error;

use stitches::spaces::LinearSpace;
use stitches::{Problem, StdoutReporter, Stitches};

struct MultiplicativePersistence;

impl Problem for MultiplicativePersistence {
    type Space = LinearSpace<u64>;
    type Out = ResultsState;

    fn check(&self, number: u64, results_state: &ResultsState) -> Option<ResultsState> {
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
    best_number: u64,
}

fn persistence(n: u64) -> u8 {
    let digits = n.to_string();
    let product: u64 = digits
        .chars()
        .map(|d| d.to_digit(10).unwrap() as u64)
        .fold(1, |acc, d| acc * d);
    if product == n {
        0
    } else {
        1 + persistence(product)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stitches = Stitches::new(MultiplicativePersistence, StdoutReporter);

    for result in stitches.results() {
        dbg!(result);
    }

    Ok(())
}
