use std::num::NonZero;

use libafl::{
    generators::Generator as _,
    inputs::{BytesInput, HasMutatorBytes},
    nonzero,
    schedulers::QueueScheduler,
    state::NopState,
};
use libafl_fandango_pyo3::{
    fandango::FandangoPythonModule,
    libafl::{FandangoGenerator, FandangoPseudoMutator},
};

use crate::{
    config::{read_corpus, FuzzerConfig},
    NUM_GENERATED,
};

pub struct FandangoConfig;

impl FuzzerConfig for FandangoConfig {
    type Mutator = FandangoPseudoMutator;

    fn mutator(opt: &crate::Opt) -> Self::Mutator {
        let module = FandangoPythonModule::new(opt.grammar_file.to_str().unwrap(), &[]).unwrap();
        FandangoPseudoMutator::new(module)
    }

    fn max_iterations() -> NonZero<usize> {
        nonzero!(1)
    }

    type Scheduler<'a> = QueueScheduler;

    fn scheduler<'a>(_observer: &super::SchedulerObserver<'a>) -> Self::Scheduler<'a> {
        QueueScheduler::new()
    }

    type Input = BytesInput;

    fn initial_inputs(_init: &mut Self::Init, opt: &crate::Opt) -> Vec<Self::Input> {
        let module = FandangoPythonModule::new(opt.grammar_file.to_str().unwrap(), &[]).unwrap();
        let mut generator = FandangoGenerator::new(module);
        let mut inputs = vec![];
        // inputs.extend((0..NUM_GENERATED).map(|_i| {
        //     generator
        //         .generate(&mut NopState::<BytesInput>::new())
        //         .unwrap()
        // }));
        inputs.extend_from_slice(
            &read_corpus()
                .iter()
                .map(|x| BytesInput::new(x.clone()))
                .collect::<Vec<_>>(),
        );
        // inputs.push(BytesInput::new(vec![]));
        inputs
        // vec![
        //     BytesInput::new(b"int main() { return 0; }".to_vec()),
        //     BytesInput::new(b";some_invalid_c_program with_more_stuff\";".to_vec()),
        //     BytesInput::new(b";some_invalid_c_program with_more_stuff\";".to_vec()),
        //     BytesInput::new(b";some_invalid_c_program with_more_stuff\";".to_vec()),
        // ]
    }

    type Init = ();

    fn init() -> Self::Init {}

    fn run_harness<'a>(_harness_setup: &'a mut Self::Init, input: &'a Self::Input) -> &'a [u8] {
        input.mutator_bytes()
    }
}
