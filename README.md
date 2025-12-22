# Compare Correctness of Inputs between Fuzzing Approches on `clang`

## Build and Run

Install [cargo](https://rustup.rs), then just (`cargo install just`), then execute the following (this may take a minute — clang is a rather complex binary to build):

```
just run
```

I have tested this on Ubuntu 22.04.5. You will probably need some additional dependencies, at least:
- clang
- git

This will download and patch clang, then build it along with the fuzzer, and finally run the fuzzer. Check the [Justfile](./Justfile) for what exactly happens.

## Configure

Check the top of the main file [`src/main.rs`](./src/main.rs), where you can specify `CurrentConfig`. Re-compile the fuzzer to use the new config. 