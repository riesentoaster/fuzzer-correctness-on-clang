use std::num::NonZero;

pub mod fandango;
pub mod nautilus;
pub mod seeds;

#[allow(unused_imports)]
pub use {fandango::FandangoConfig, nautilus::NautilusConfig};

use libafl::{
    inputs::BytesInput,
    observers::{ObserversTuple, StdErrObserver, StdOutObserver},
    Error,
};
use libafl_bolts::shmem::ShMemDescription;

use crate::{config::seeds::SeedsConfig, Opt};

pub type SchedulerObserver<'a> = libafl::observers::ExplicitTracking<
    libafl::observers::HitcountsMapObserver<libafl::observers::StdMapObserver<'a, u8, false>>,
    true,
    false,
>;

pub trait FuzzerConfig<Seeds: SeedsConfig> {
    type Mutator;
    type Scheduler<'a>;
    type Input;
    type Init;
    type Executor<'a, OT, S>;
    fn mutator(opt: &Opt) -> Self::Mutator;
    fn max_iterations() -> NonZero<usize>;
    fn scheduler<'a>(observer: &SchedulerObserver<'a>) -> Self::Scheduler<'a>;
    fn initial_inputs(init: &mut Self::Init, opt: &Opt) -> Vec<Self::Input>;
    fn init() -> Self::Init;
    fn get_executor<'a, OT: ObserversTuple<BytesInput, S>, S>(
        init: &'a mut Self::Init,
        stdout_observer: StdOutObserver,
        stderr_observer: StdErrObserver,
        observers: OT,
        shmem_description: ShMemDescription,
    ) -> Result<Self::Executor<'a, OT, S>, Error>;
}
