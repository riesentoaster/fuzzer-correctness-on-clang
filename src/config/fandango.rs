use std::{marker::PhantomData, num::NonZero};

use libafl::{
    inputs::BytesInput, nonzero, observers::ObserversTuple, schedulers::QueueScheduler, Error,
};
use libafl_fandango_pyo3::{fandango::FandangoPythonModule, libafl::FandangoPseudoMutator};

use crate::{
    config::{seeds::SeedsConfig, FuzzerConfig},
    executor::{get_executor, GenericExecutor},
};

pub struct FandangoConfig<Seeds: SeedsConfig>(PhantomData<Seeds>);

impl<Seeds: SeedsConfig> FuzzerConfig<Seeds> for FandangoConfig<Seeds> {
    type Mutator = FandangoPseudoMutator;

    fn mutator(opt: &crate::Opt) -> Self::Mutator {
        let module =
            FandangoPythonModule::new(format!("{}.fan", opt.grammar_file_prefix).as_str(), &[])
                .unwrap();
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

    fn initial_inputs(_init: &mut Self::Init, _opt: &crate::Opt) -> Vec<Self::Input> {
        let mut inputs = vec![];
        inputs.extend_from_slice(
            &Seeds::get_seeds()
                .iter()
                .map(|x| BytesInput::new(x.clone()))
                .collect::<Vec<_>>(),
        );
        if inputs.is_empty() {
            inputs.push(BytesInput::new(vec![]));
        }
        inputs
    }

    type Init = ();

    fn init() -> Self::Init {}

    type Executor<'a, OT, S> = GenericExecutor<BytesInput, OT, S>;

    fn get_executor<'a, OT: ObserversTuple<BytesInput, S>, S>(
        _init: &'a mut Self::Init,
        stdout_observer: libafl::observers::StdOutObserver,
        stderr_observer: libafl::observers::StdErrObserver,
        observers: OT,
        shmem_description: libafl_bolts::shmem::ShMemDescription,
    ) -> Result<Self::Executor<'a, OT, S>, Error> {
        get_executor(
            stdout_observer,
            stderr_observer,
            observers,
            shmem_description,
        )
    }
}
