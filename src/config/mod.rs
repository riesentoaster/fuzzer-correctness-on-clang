use std::{fs, num::NonZero, path::Path};

pub mod fandango;
pub mod nautilus;
pub use fandango::FandangoConfig;
pub use nautilus::NautilusConfig;

use crate::Opt;

pub type SchedulerObserver<'a> = libafl::observers::ExplicitTracking<
    libafl::observers::HitcountsMapObserver<libafl::observers::StdMapObserver<'a, u8, false>>,
    true,
    false,
>;

pub trait FuzzerConfig {
    type Mutator;
    type Scheduler<'a>;
    type Input;
    type Init;
    fn mutator(opt: &Opt) -> Self::Mutator;
    fn max_iterations() -> NonZero<usize>;
    fn scheduler<'a>(observer: &SchedulerObserver<'a>) -> Self::Scheduler<'a>;
    fn initial_inputs(init: &mut Self::Init, opt: &Opt) -> Vec<Self::Input>;
    fn init() -> Self::Init;
    fn run_harness<'a>(init: &'a mut Self::Init, input: &'a Self::Input) -> &'a [u8];
}

pub fn read_corpus() -> Vec<Vec<u8>> {
    let corpus_dir = Path::new("valid_corpus");
    let mut inputs = vec![];
    let mut read_something = false;
    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(bytes) = fs::read(&path) {
                    read_something = true;
                    inputs.push(bytes);
                }
            }
        }
    }
    assert!(read_something);
    inputs
}
