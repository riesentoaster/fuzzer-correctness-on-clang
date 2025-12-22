COVERAGE_FILE := "target/release/coverage.o"

[unix]
coverage_collector:
    clang -c -o {{COVERAGE_FILE}} coverage.c

[unix]
preloads:
    cargo build --release --package get_guard_num
    cargo build --release --package setup_guard_redirection

[unix]
source:
    if [ ! -f llvm/README.md ]; then \
        rm -rf llvm && \
        git clone https://github.com/llvm/llvm-project llvm && \
        cd llvm && \
        git checkout 72c69aefbae8 && \
        git apply ../clang.diff && \
        cd .. || (echo "Failed to initialize and patch LLVM" && exit 1); \
    fi

[unix]
build: source coverage_collector
    cd llvm && \
    mkdir -p build && cd build && \
    cmake -GNinja -DCMAKE_BUILD_TYPE=Release ../llvm \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DLLVM_ENABLE_PROJECTS="clang;lld;clang-tools-extra" \
    -DLLVM_ENABLE_RUNTIMES="libcxx;libcxxabi;compiler-rt" \
    -DCMAKE_C_FLAGS="-fsanitize-coverage=trace-pc-guard" \
    -DCMAKE_CXX_FLAGS="-fsanitize-coverage=trace-pc-guard" \
    -DCMAKE_EXE_LINKER_FLAGS="$(realpath ../../{{COVERAGE_FILE}})" \
    -DLLVM_ENABLE_ASSERTIONS=ON && \
    ninja clang -j $(nproc);

[unix]
fuzzer:
    cargo build --release

run: fuzzer build preloads
    ./target/release/libafl_nautilus_fuzzer \
    --grammar-file c.fan

run_fast: fuzzer preloads
    ./target/release/libafl_nautilus_fuzzer \
    --grammar-file c.fan
