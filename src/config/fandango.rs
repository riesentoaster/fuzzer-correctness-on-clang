use std::marker::PhantomData;

use libafl::{inputs::BytesInput, observers::ObserversTuple, schedulers::QueueScheduler, Error};

use crate::{
    config::{seeds::SeedsConfig, FuzzerConfig},
    executor::{get_executor, GenericExecutor},
};

#[allow(unused)]
pub struct FandangoConfig<Seeds: SeedsConfig>(PhantomData<Seeds>);

impl<Seeds: SeedsConfig> FuzzerConfig<Seeds> for FandangoConfig<Seeds> {
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

#[allow(unused_macros)]
macro_rules! setup_fandango_stages {
    ($opt:expr, $inner:expr, $min:expr, $max:expr) => {{
        let module = libafl_fandango_pyo3::fandango::FandangoPythonModule::new(
            &format!("{}.fan", $opt.grammar_file_prefix),
            &[],
        )
        .unwrap();
        tuple_list!(
            libafl_fandango_pyo3::libafl::FandangoPostMutationalStage::new(
                module, $inner, $min, $max
            )
        )
    }};
}
