use std::error::Error;

use stitches::spaces::LinearSpace;
use stitches::{Problem, StdoutReporter, Stitches};

struct Null;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Number(Option<usize>);

impl Problem for Null {
    // Uncomment for limiting time for profiling.
    // type Space = stitches::spaces::TimeLimited<LinearSpace<usize>>;
    type Space = LinearSpace<usize>;
    type Out = Number;

    fn initial_space(&mut self) -> Self::Space {
        // Uncomment for limiting time for profiling.
        // stitches::spaces::TimeLimited::new(
        //     std::time::Duration::from_secs(10),
        //     LinearSpace::default(),
        // )
        LinearSpace::default()
    }

    fn check(&self, number: usize, last_out: &Number) -> Option<Number> {
        if bencher::black_box(number) == std::usize::MAX {
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
