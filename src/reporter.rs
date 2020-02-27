use crate::Problem;

pub trait Reporter<P: Problem> {
    fn report_on(&mut self, space: &P::Space, out: &P::Out);
}

pub struct NullReporter;

impl<P: Problem> Reporter<P> for NullReporter {
    fn report_on(&mut self, space: &P::Space, out: &P::Out) {}
}

pub struct StdoutReporter;

impl<P: Problem> Reporter<P> for StdoutReporter
where
    P::Space: core::fmt::Debug,
    P::Out: core::fmt::Debug,
{
    fn report_on(&mut self, space: &P::Space, out: &P::Out) {
        println!("space: {:?}", space);
        println!("out: {:?}", out);
    }
}
