use std::error::Error;

use stitches::spaces::LinearSpace;
use stitches::{Problem, StdoutReporter, Stitches};

struct Null;

#[derive(Clone, Copy, Debug, Default)]
struct Number(Option<u64>);

impl Problem for Null {
    type Space = LinearSpace<u64>;
    type Out = Number;

    fn initial_space(&mut self) -> Self::Space {
        LinearSpace::new(4125929382)
    }

    fn check(&self, number: u64, last_out: &Number) -> Option<Number> {
        if number == std::u64::MAX {
            Some(Number(Some(number)))
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stitches = Stitches::new(Null, StdoutReporter);
    for result in stitches.results() {
        dbg!(result);
    }
    Ok(())
}
