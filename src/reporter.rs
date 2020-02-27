use crate::{Problem, Stats};

pub trait Reporter<P: Problem> {
    fn report_on(&mut self, space: &P::Space, out: &P::Out, stats: &Stats);
}

pub struct NullReporter;

impl<P: Problem> Reporter<P> for NullReporter {
    fn report_on(&mut self, _: &P::Space, _: &P::Out, _: &Stats) {}
}

pub struct StdoutReporter;

impl<P: Problem> Reporter<P> for StdoutReporter
where
    P::Space: core::fmt::Debug,
    P::Out: core::fmt::Debug,
{
    fn report_on(&mut self, space: &P::Space, out: &P::Out, stats: &Stats) {
        println!("space: {:?}", space);
        println!("out: {:?}", out);
        println!(
            "throughput: {}/s",
            human_format::Formatter::new().format(stats.throughput())
        );
    }
}
