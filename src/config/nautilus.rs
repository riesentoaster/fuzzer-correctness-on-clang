use std::{fs, io::Write as _, num::NonZero};

use libafl::{
    executors::{Executor, HasObservers},
    generators::{Generator as _, NautilusContext, NautilusGenerator},
    inputs::{
        BytesInput, EncodedInput, InputDecoder as _, InputEncoder as _, NaiveTokenizer,
        NautilusInput, TokenInputEncoderDecoder,
    },
    mutators::{
        encoded_mutations, EncodedAddMutator, EncodedCopyMutator, EncodedCrossoverInsertMutator,
        EncodedCrossoverReplaceMutator, EncodedDecMutator, EncodedDeleteMutator, EncodedIncMutator,
        EncodedInsertCopyMutator, EncodedRandMutator, HavocScheduledMutator,
    },
    nonzero,
    observers::ObserversTuple,
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    stages::mutational::DEFAULT_MUTATIONAL_MAX_ITERATIONS,
    state::NopState,
};
use libafl_bolts::tuples::{tuple_list_type, RefIndexable};

use crate::{
    config::{read_corpus, FuzzerConfig},
    Opt, NUM_GENERATED,
};

pub struct NautilusConfig;

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

type SchedulerObserver<'a> = libafl::observers::ExplicitTracking<
    libafl::observers::HitcountsMapObserver<libafl::observers::StdMapObserver<'a, u8, false>>,
    true,
    false,
>;

impl FuzzerConfig for NautilusConfig {
    type Mutator = HavocScheduledMutator<EncodedMutations>;

    fn mutator(_opt: &crate::Opt) -> Self::Mutator {
        HavocScheduledMutator::with_max_stack_pow(encoded_mutations(), 2)
    }

    fn max_iterations() -> NonZero<usize> {
        nonzero!(DEFAULT_MUTATIONAL_MAX_ITERATIONS)
    }

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

        let context = NautilusContext::from_file(256, opt.grammar_file.clone()).unwrap();
        let mut tokenizer = NaiveTokenizer::default();
        let mut initial_inputs = vec![];
        let mut generator = NautilusGenerator::new(&context);

        let mut bytes = vec![];
        for i in 0..NUM_GENERATED {
            let nautilus = generator
                .generate(&mut NopState::<NautilusInput>::new())
                .unwrap();
            nautilus.unparse(&context, &mut bytes);

            let mut file = fs::File::create(initial_dir.join(format!("id_{i}"))).unwrap();
            file.write_all(&bytes).unwrap();

            let input = encoder_decoder
                .encode(&bytes, &mut tokenizer)
                .expect("encoding failed");
            initial_inputs.push(input);
        }
        // initial_inputs.extend_from_slice(
        //     &read_corpus()
        //         .iter()
        //         .flat_map(|x| encoder_decoder.encode(x, &mut tokenizer))
        //         .collect::<Vec<_>>(),
        // );
        initial_inputs
    }

    type Init = (Vec<u8>, TokenInputEncoderDecoder);

    fn init() -> Self::Init {
        (vec![], TokenInputEncoderDecoder::new())
    }

    fn run_harness<'a>(init: &'a mut Self::Init, input: &'a Self::Input) -> &'a [u8] {
        let (ref mut bytes, ref mut encoder_decoder) = init;
        bytes.clear();
        encoder_decoder.decode(input, bytes).unwrap();
        if *bytes.last().unwrap() != 0 {
            bytes.push(0);
        }
        bytes.as_slice()
    }
}

pub struct NautilusUnparsingExecutor<'a, E> {
    init: &'a mut <NautilusConfig as FuzzerConfig>::Init,
    inner: E,
}

impl<'a, E> NautilusUnparsingExecutor<'a, E> {
    pub fn new(init: &'a mut <NautilusConfig as FuzzerConfig>::Init, inner: E) -> Self {
        Self { init, inner }
    }
}

impl<'a, E> HasObservers for NautilusUnparsingExecutor<'a, E>
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

impl<'a, E, EM, S, Z> Executor<EM, EncodedInput, S, Z> for NautilusUnparsingExecutor<'a, E>
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
        let unparsed_input = NautilusConfig::run_harness(self.init, input);
        self.inner.run_target(
            fuzzer,
            state,
            mgr,
            &BytesInput::new(unparsed_input.to_vec()),
        )
    }
}
