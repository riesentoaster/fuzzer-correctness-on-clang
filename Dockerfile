FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    clang \
    curl \
    ninja-build \
    python3-dev \
    python3-pip


RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN pip install --break-system-packages fandango-fuzzer

RUN cargo install just

COPY Justfile clang.diff coverage.c ./
# cache this step
RUN just build
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN just preloads fuzzer

COPY c.fan c.json /
COPY valid_corpus /valid_corpus

CMD ["just", "run_fast"]