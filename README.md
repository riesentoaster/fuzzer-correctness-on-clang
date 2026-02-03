# Compare Correctness of Inputs between Fuzzing Approches on `clang`

## Report

This is the fuzzing infrastructure behind my short paper accepted at the Doctoral Symposium at ICSE 2026. You can find the submission PDF [here](./report/Valentin-Huber-Testing-Programs-Expecting-Highly-Constrained-Inputs-ICSE-2026-DS.pdf).

## Build and Run

Check out the [`Dockerfile`](./Dockerfile) for installation and usage instructions. To build and run:

```bash
docker build -t fuzzer-correctness .
docker run --rm fuzzer-correctness
```

## Configure

Check the section in [`src/main.rs`](./src/main.rs), where you can specify `CurrentConfig`. Adjust the macro invocation as needed. Re-compile the fuzzer to use the new config. You may use the scripts [`run_in_screen.sh`](./run_in_screen.sh) and [`run.sh`](./run.sh) as well.

## Output

Output from some runs can be found in the [`out`](./out) directory. Configuration was as follows:
- fandango: Using Fandango via `libafl-fandango-pyo3` as a grammar-based fuzzer (grammar in [`c.fan`](./c.fan)). While Fandango supports semantic constraints across derivation trees, this was not used here – fandango was used as a pure grammar fuzzer
- nautilus: Using Nautilus 2.0 with its LibAFL integration (grammar in [`c.json`](./c.json)). Nautilus is a coverage-guided grammar-based fuzzer.
- havoc: Building a simple coverage-guided byte-mutating fuzzer, in the same ballpark as AFL++ (uses the Fandango Config, but create a simple mutational stage with all havoc mutations, so doesn't actually call Fandango).
- fandango-posthavoc: Similar to the first option, but each output produced by Fandango is first fed to the target and then then mutated `n` times using non-crossover havoc mutations (each time starting from the unaltered Fandango-produced input).
- fandango-interspersedhavoc: Similar to the first option, but with an additional muatational stage using all havoc mutations. This is essentially equivalent to an AFL++-style fuzzer which will every once in a while also call Fandango to create new inputs from scratch.

## Plots

Check out the script used to create the plots in [`analyze.py`](./analyze.py).

You can find its output in [`plots`](./plots).
