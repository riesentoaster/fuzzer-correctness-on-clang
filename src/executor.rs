use std::{
    path::Path,
    process::{Child, Command},
    time::Duration,
};

use libafl::{
    executors::{command::StdCommandConfigurator, CommandExecutor, StdChildArgs},
    inputs::HasTargetBytes,
    observers::{ObserversTuple, StdErrObserver, StdOutObserver},
    Error,
};

use libafl_bolts::{
    merge_tuple_list_type,
    shmem::ShMemDescription,
    tuples::{tuple_list, tuple_list_type, Handled as _, Merge as _},
    StdTargetArgs,
};

use crate::TARGET_BINARY;

pub type GenericExecutor<I, OT, S> = CommandExecutor<
    Child,
    (),
    I,
    merge_tuple_list_type!(tuple_list_type!(StdOutObserver, StdErrObserver), OT),
    S,
    StdCommandConfigurator,
>;

pub fn get_executor<I: HasTargetBytes, OT: ObserversTuple<I, S>, S>(
    stdout_observer: StdOutObserver,
    stderr_observer: StdErrObserver,
    observers: OT,
    shmem_description: ShMemDescription,
) -> Result<GenericExecutor<I, OT, S>, Error> {
    let shmem_description_string = serde_json::to_string(&shmem_description).unwrap();

    let stdout = stdout_observer.handle();
    let stderr = stderr_observer.handle();

    CommandExecutor::builder()
        .program(TARGET_BINARY)
        .env(
            "LD_PRELOAD",
            "./target/release/libsetup_guard_redirection.so",
        )
        .env("SHMEM_DESCRIPTION", shmem_description_string)
        .stdout_observer(stdout.clone())
        .stderr_observer(stderr.clone())
        .args(["-o", "/dev/null", "-xc++", "-fintegrated-cc1", "-"])
        .timeout(Duration::from_secs(1))
        .build(tuple_list!(stdout_observer, stderr_observer).merge(observers))
}

pub fn get_coverage_shmem_size(binary: &str) -> Result<usize, Error> {
    if !Path::new(&binary).exists() {
        return Err(Error::illegal_argument(format!(
            "Binary {binary} not found"
        )));
    }

    let shared = "./target/release/libget_guard_num.so";
    if !Path::new(shared).exists() {
        return Err(Error::illegal_argument(
        "Missing shared library to instrument binary to find number of edges. Check Makefile.toml for the appropriate target."
        ));
    }

    let guard_num_command_output = Command::new(binary)
        .env("LD_PRELOAD", shared)
        .output()?
        .stdout;

    let guard_num_command_output_string = String::from_utf8(guard_num_command_output)?;
    let guard_num = guard_num_command_output_string.trim().parse::<usize>()?;

    match guard_num {
        0 => Err(Error::illegal_state("Binary reported a guard count of 0")),
        e => Ok(e),
    }
}
