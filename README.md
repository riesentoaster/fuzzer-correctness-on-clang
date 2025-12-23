# Compare Correctness of Inputs between Fuzzing Approches on `clang`

## Build and Run

Check out the [`Dockerfile`](./Dockerfile) for installation and usage instructions. To build and run:

```bash
docker build -t fuzzer-correctness .
docker run --rm fuzzer-correctness
```

## Configure

Check the top of the main file [`src/main.rs`](./src/main.rs), where you can specify `CurrentConfig`. Re-compile the fuzzer to use the new config. 