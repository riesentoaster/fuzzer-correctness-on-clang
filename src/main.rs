//! A libfuzzer-like fuzzer with llmp-multithreading support and restarts
//! The example harness is built for libpng.
//! This will fuzz javascript.

mod config;
mod executor;
mod feedback;
mod observer;

use clap::Parser;
use core::time::Duration;
use std::{borrow::Cow, env, fs, net::SocketAddr, path::PathBuf};

use libafl::{
    corpus::{CachedOnDiskCorpus, OnDiskCorpus},
    events::{
        ClientDescription, EventConfig, EventRestarter, Launcher, LlmpRestartingEventManager,
    },
    feedback_or, feedback_or_fast,
    feedbacks::{
        stdio::{StdErrToMetadataFeedback, StdOutToMetadataFeedback},
        CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback,
    },
    fuzzer::{Evaluator, Fuzzer},
    monitors::{MultiMonitor, OnDiskJsonMonitor},
    observers::{
        CanTrack, HitcountsMapObserver, StdErrObserver, StdMapObserver, StdOutObserver,
        TimeObserver,
    },
    stages::mutational::StdMutationalStage,
    state::StdState,
    BloomInputFilter, Error, ReportingInputFilter, StdFuzzer,
};
use libafl_bolts::{
    core_affinity::Cores,
    current_nanos,
    rands::StdRand,
    shmem::{MmapShMemProvider, ShMem as _, ShMemProvider, StdShMemProvider},
    tuples::tuple_list,
    AsSliceMut as _,
};

use observer::CorrectnessObserver;

use crate::{
    config::{seeds::ValidCorpusSeedsConfig, FuzzerConfig},
    executor::get_coverage_shmem_size,
    feedback::ReportCorrectnessFeedback,
};

/// Parses a millseconds int into a [`Duration`], used for commandline arg parsing
fn timeout_from_millis_str(time: &str) -> Result<Duration, Error> {
    Ok(Duration::from_millis(time.parse()?))
}

#[derive(Debug, Parser)]
#[command(
    name = "fuzzer_correctness_on_clang",
    about = "Compare the correctness of inputs produced by different fuzzing approaches on clang",
    author = "Valentin Huber <contact@valentinhuber.me>"
)]
pub struct Opt {
    #[arg(
        short,
        long,
        value_parser = Cores::from_cmdline,
        help = "Spawn a client in each of the provided cores. Broker runs in the 0th core. 'all' to select all available cores. 'none' to run a client without binding to any core. eg: '1,2-4,6' selects the cores 1,2,3,4,6.",
        name = "CORES",
        default_value = "0"
    )]
    cores: Cores,

    #[arg(
        short = 'p',
        long,
        help = "Choose the broker TCP port, default is 1337",
        name = "PORT",
        default_value = "1337"
    )]
    broker_port: u16,

    #[arg(short = 'a', long, help = "Specify a remote broker", name = "REMOTE")]
    remote_broker_addr: Option<SocketAddr>,

    #[arg(
        short,
        long,
        help = "Set the output directory, default is ./out",
        name = "OUTPUT",
        default_value = "./out"
    )]
    output: PathBuf,

    #[arg(
        value_parser = timeout_from_millis_str,
        short,
        long,
        help = "Set the execution timeout in milliseconds, default is 1000",
        name = "TIMEOUT",
        default_value = "1000"
    )]
    timeout: Duration,

    #[arg(long, help = "Set the stdout file", name = "STDOUT_FILE")]
    stdout_file: Option<PathBuf>,

    #[arg(long, help = "Set the stderr file", name = "STDERR_FILE")]
    stderr_file: Option<PathBuf>,

    #[arg(short, long, help = "Set the grammar file", name = "GRAMMAR_FILE")]
    grammar_file_prefix: String,
}

static TARGET_BINARY: &str = "./llvm/build/bin/clang";

const NUM_GENERATED: usize = 4096;
const CORPUS_CACHE: usize = 4096;

type CurrentConfig = config::FandangoConfig<ValidCorpusSeedsConfig>;

#[allow(clippy::too_many_lines)]
pub fn main() {
    let opt = Opt::parse();

    let mut initial_dir = opt.output.clone();
    initial_dir.push("initial");
    fs::create_dir_all(&initial_dir).unwrap();

    println!(
        "Workdir: {:?}",
        env::current_dir().unwrap().to_string_lossy().to_string()
    );

    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");
    let stats = OnDiskJsonMonitor::new(opt.output.join("stats.json"), |_| true);
    let print = MultiMonitor::new(|s| println!("{s}"));
    let monitor = tuple_list!(stats, print);

    let mut run_client = |state: Option<StdState<_, _, _, _>>,
                          mut restarting_mgr: LlmpRestartingEventManager<_, _, _, _, _>,
                          core_id: ClientDescription| {
        let mut objective_dir = opt.output.clone();
        objective_dir.push("crashes");
        let mut corpus_dir = opt.output.clone();
        corpus_dir.push(format!(
            "corpus_{}_{}",
            core_id.core_id().0,
            core_id.overcommit_id()
        ));

        #[allow(clippy::let_unit_value)]
        let mut init = CurrentConfig::init();
        let initial_inputs = CurrentConfig::initial_inputs(&mut init, &opt);

        let guard_num = get_coverage_shmem_size(TARGET_BINARY)?;

        let mut provider = MmapShMemProvider::default();
        let mut shmem = provider
            .new_shmem(guard_num + size_of::<usize>())?
            .persist()?;
        let shmem_description = shmem.description();

        let (step, edges) = shmem.as_slice_mut().split_at_mut(size_of::<usize>());

        let edges_observer =
            HitcountsMapObserver::new(unsafe { StdMapObserver::new("edges", edges) })
                .track_indices();

        // Create an observation channel to keep track of the execution time
        let time_observer = TimeObserver::new("time");

        // Custom correctness observer backed by a global no_mangle symbol
        let correctness_observer =
            CorrectnessObserver::new(step, format!("correctness_{}", core_id.core_id().0));

        let stdout_observer = StdOutObserver::new(Cow::Borrowed("stdout")).unwrap();
        let stderr_observer = StdErrObserver::new(Cow::Borrowed("stderr")).unwrap();

        let stdout_feedback = StdOutToMetadataFeedback::new(&stdout_observer);
        let stderr_feedback = StdErrToMetadataFeedback::new(&stderr_observer);

        // Feedback to rate the interestingness of an input
        // This one is composed by two Feedbacks in OR
        let mut feedback = feedback_or!(
            stdout_feedback.clone(),
            stderr_feedback.clone(),
            ReportCorrectnessFeedback::new(&correctness_observer),
            // New maximization map feedback linked to the edges observer and the feedback state
            MaxMapFeedback::new(&edges_observer),
            // Time feedback, this one does not need a feedback state
            TimeFeedback::new(&time_observer)
        );

        // A feedback to choose if an input is a solution or not
        let mut objective = feedback_or_fast!(
            stdout_feedback,
            stderr_feedback,
            CrashFeedback::new(),
            TimeoutFeedback::new(),
        );

        // If not restarting, create a State from scratch
        let mut state = state.unwrap_or_else(|| {
            StdState::new(
                // RNG
                StdRand::with_seed(current_nanos()),
                // Corpus that will be evolved, we keep it in memory for performance
                CachedOnDiskCorpus::new(corpus_dir, CORPUS_CACHE).unwrap(),
                // Corpus in which we store solutions (crashes in this example),
                // on disk so the user can get them after stopping the fuzzer
                OnDiskCorpus::new(objective_dir).unwrap(),
                &mut feedback,
                &mut objective,
            )
            .unwrap()
        });

        // A minimization+queue policy to get testcasess from the corpus
        let scheduler = CurrentConfig::scheduler(&edges_observer);

        // A fuzzer with feedbacks and a corpus scheduler
        let mut fuzzer = StdFuzzer::builder()
            .input_filter(ReportingInputFilter::new(BloomInputFilter::default(), 100))
            .scheduler(scheduler)
            .feedback(feedback)
            .objective(objective)
            .build();

        // // The wrapped harness function, calling out to the LLVM-style harness
        // let mut harness = |input: &<CurrentConfig as FuzzerConfig>::Input| {
        //     let bytes = CurrentConfig::run_harness(&mut init, input);

        //     //unsafe {
        //     //println!(">>> {}", std::str::from_utf8_unchecked(&bytes));
        //     //}
        //     unsafe { libfuzzer_test_one_input(bytes) };
        //     ExitKind::Ok
        // };

        // // Create the executor for an in-process function with one observer for edge coverage and one for the execution time
        // let mut executor = InProcessExecutor::with_timeout(
        //     &mut harness,
        //     tuple_list!(edges_observer, time_observer, correctness_observer),
        //     &mut fuzzer,
        //     &mut state,
        //     &mut restarting_mgr,
        //     opt.timeout,
        // )?;

        let mut executor = CurrentConfig::get_executor(
            &mut init,
            stdout_observer,
            stderr_observer,
            tuple_list!(edges_observer, time_observer, correctness_observer),
            shmem_description,
        )?;

        // let mut executor = NautilusUnparsingExecutor::new(&mut init, executor);

        // The actual target run starts here.
        // Call LLVMFUzzerInitialize() if present.
        // let args: Vec<String> = env::args().collect();
        // if unsafe { libfuzzer_initialize(&args) } == -1 {
        //     println!("Warning: LLVMFuzzerInitialize failed with -1");
        // }

        // In case the corpus is empty (on first run), reset
        if state.must_load_initial_inputs() {
            println!("Loading {} initial inputs", initial_inputs.len());
            for input in &initial_inputs {
                fuzzer
                    .add_input(
                        &mut state,
                        &mut executor,
                        &mut restarting_mgr,
                        input.clone(),
                    )
                    .unwrap();
            }
        }

        // Setup a basic mutator with a mutational stage
        let mut stages = tuple_list!(
            StdMutationalStage::with_max_iterations(
                CurrentConfig::mutator(&opt),
                CurrentConfig::max_iterations()
            ),
            // StdMutationalStage::new(HavocScheduledMutator::new(havoc_mutations())) // StdMutationalStage::new(NopMutator::new(libafl::mutators::MutationResult::Mutated))
        );

        println!("Let's fuzz!");
        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut restarting_mgr)?;
        restarting_mgr.on_restart(&mut state)
    };

    println!("launching launcher");

    // Set default stdout/stderr files if not provided
    let stdout_file = opt.stdout_file.as_ref().map(|p| {
        let mut outer = opt.output.clone();
        outer.push(p);
        outer.to_string_lossy().to_string()
    });

    let stderr_file = opt.stderr_file.as_ref().map(|p| {
        let mut outer = opt.output.clone();
        outer.push(p);
        outer.to_string_lossy().to_string()
    });

    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .configuration(EventConfig::AlwaysUnique)
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&opt.cores)
        .broker_port(opt.broker_port)
        // .remote_broker_addr(opt.remote_broker_addr)
        .stdout_file(stdout_file.as_deref())
        .stderr_file(stderr_file.as_deref())
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}
