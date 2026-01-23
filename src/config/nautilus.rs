use std::{fs, marker::PhantomData};

use libafl::{
    executors::{Executor, HasObservers},
    generators::{Generator as _, NautilusContext, NautilusGenerator},
    inputs::{
        BytesInput, EncodedInput, InputDecoder as _, InputEncoder as _, NaiveTokenizer,
        NautilusInput, TokenInputEncoderDecoder,
    },
    mutators::{
        EncodedAddMutator, EncodedCopyMutator, EncodedCrossoverInsertMutator,
        EncodedCrossoverReplaceMutator, EncodedDecMutator, EncodedDeleteMutator, EncodedIncMutator,
        EncodedInsertCopyMutator, EncodedRandMutator,
    },
    observers::ObserversTuple,
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    state::NopState,
    Error,
};
use libafl_bolts::tuples::{tuple_list_type, RefIndexable};

use crate::{
    config::{seeds::SeedsConfig, FuzzerConfig},
    executor::{get_executor, GenericExecutor},
    Opt, NUM_GENERATED,
};

#[allow(unused)]
pub struct NautilusConfig<Seeds: SeedsConfig>(PhantomData<Seeds>);

#[allow(unused)]
type EncodedMutations = tuple_list_type!(
    EncodedRandMutator,
    EncodedIncMutator,
    EncodedDecMutator,
    EncodedAddMutator,
    EncodedDeleteMutator,
    EncodedInsertCopyMutator,
    EncodedCopyMutator,
    EncodedCrossoverInsertMutator,
    EncodedCrossoverReplaceMutator,
);

#[allow(unused)]
type SchedulerObserver<'a> = libafl::observers::ExplicitTracking<
    libafl::observers::HitcountsMapObserver<libafl::observers::StdMapObserver<'a, u8, false>>,
    true,
    false,
>;

impl<Seeds: SeedsConfig> FuzzerConfig<Seeds> for NautilusConfig<Seeds> {
    type Scheduler<'a> = libafl::schedulers::MinimizerScheduler<
        QueueScheduler,
        libafl::schedulers::LenTimeMulTestcasePenalty,
        EncodedInput,
        libafl::feedbacks::MapIndexesMetadata,
        SchedulerObserver<'a>,
    >;

    fn scheduler<'a>(observer: &SchedulerObserver<'a>) -> Self::Scheduler<'a> {
        IndexesLenTimeMinimizerScheduler::new(observer, QueueScheduler::new())
    }

    type Input = EncodedInput;

    fn initial_inputs(init: &mut Self::Init, opt: &Opt) -> Vec<Self::Input> {
        let (ref mut _bytes, ref mut encoder_decoder) = init;
        let mut initial_dir = opt.output.clone();
        initial_dir.push("initial");
        fs::create_dir_all(&initial_dir).unwrap();

        let context =
            NautilusContext::from_file(256, format!("{}.json", opt.grammar_file_prefix)).unwrap();
        let mut tokenizer = NaiveTokenizer::default();
        let mut initial_inputs = vec![];
        let mut generator = NautilusGenerator::new(&context);

        let mut bytes = vec![];
        for _i in 0..NUM_GENERATED {
            let nautilus = generator
                .generate(&mut NopState::<NautilusInput>::new())
                .unwrap();
            nautilus.unparse(&context, &mut bytes);

            // let mut file = fs::File::create(initial_dir.join(format!("id_{i}"))).unwrap();
            // file.write_all(&bytes).unwrap();

            let input = encoder_decoder
                .encode(&bytes, &mut tokenizer)
                .expect("encoding failed");
            initial_inputs.push(input);
        }
        initial_inputs.extend_from_slice(
            &Seeds::get_seeds()
                .iter()
                .flat_map(|x| encoder_decoder.encode(x, &mut tokenizer))
                .collect::<Vec<_>>(),
        );
        initial_inputs
    }

    type Init = (Vec<u8>, TokenInputEncoderDecoder);

    fn init() -> Self::Init {
        (vec![], TokenInputEncoderDecoder::new())
    }

    type Executor<'a, OT, S> =
        NautilusUnparsingExecutor<'a, GenericExecutor<BytesInput, OT, S>, Seeds>;

    fn get_executor<'a, OT: ObserversTuple<BytesInput, S>, S>(
        init: &'a mut Self::Init,
        stdout_observer: libafl::observers::StdOutObserver,
        stderr_observer: libafl::observers::StdErrObserver,
        observers: OT,
        shmem_description: libafl_bolts::shmem::ShMemDescription,
    ) -> Result<Self::Executor<'a, OT, S>, Error> {
        let inner = get_executor(
            stdout_observer,
            stderr_observer,
            observers,
            shmem_description,
        )?;
        Ok(NautilusUnparsingExecutor::new(init, inner))
    }
}

#[allow(unused)]
pub struct NautilusUnparsingExecutor<'a, E, Seeds: SeedsConfig> {
    init: &'a mut <NautilusConfig<Seeds> as FuzzerConfig<Seeds>>::Init,
    inner: E,
}

impl<'a, E, Seeds: SeedsConfig> NautilusUnparsingExecutor<'a, E, Seeds> {
    #[allow(unused)]
    pub fn new(
        init: &'a mut <NautilusConfig<Seeds> as FuzzerConfig<Seeds>>::Init,
        inner: E,
    ) -> Self {
        Self { init, inner }
    }
}

impl<'a, E, Seeds: SeedsConfig> HasObservers for NautilusUnparsingExecutor<'a, E, Seeds>
where
    E: HasObservers,
{
    type Observers = E::Observers;

    fn observers(&self) -> RefIndexable<&Self::Observers, Self::Observers> {
        self.inner.observers()
    }

    fn observers_mut(&mut self) -> RefIndexable<&mut Self::Observers, Self::Observers> {
        self.inner.observers_mut()
    }
}

impl<'a, E, EM, S, Z, Seeds: SeedsConfig> Executor<EM, EncodedInput, S, Z>
    for NautilusUnparsingExecutor<'a, E, Seeds>
where
    E: Executor<EM, BytesInput, S, Z>,
{
    fn run_target(
        &mut self,
        fuzzer: &mut Z,
        state: &mut S,
        mgr: &mut EM,
        input: &EncodedInput,
    ) -> Result<libafl::executors::ExitKind, libafl::Error> {
        let (ref mut bytes, ref mut encoder_decoder) = self.init;
        bytes.clear();
        encoder_decoder.decode(input, bytes).unwrap();
        if *bytes.last().unwrap() != 0 {
            bytes.push(0);
        }

        let unparsed_input = bytes.as_slice();
        self.inner.run_target(
            fuzzer,
            state,
            mgr,
            &BytesInput::new(unparsed_input.to_vec()),
        )
    }
}

#[allow(unused_macros)]
macro_rules! setup_nautilus_stages {
    ($opt:expr) => {
        tuple_list!(libafl::stages::mutational::StdMutationalStage::new(
            libafl::mutators::HavocScheduledMutator::with_max_stack_pow(
                libafl::mutators::encoded_mutations(),
                2
            )
        ))
    };
}
