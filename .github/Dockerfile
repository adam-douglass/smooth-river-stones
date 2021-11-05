# FROM rust:1-slim

# RUN cargo install trunk wasm-bindgen-cli -j 4
# RUN rustup target add wasm32-unknown-unknown

FROM debian:11-slim

RUN apt-get update && apt-get install -y curl gcc git rsync pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf > rustup-init.sh && \
    chmod +x rustup-init.sh && \
    /rustup-init.sh -y --default-toolchain stable --profile minimal -t wasm32-unknown-unknown && \
    rm rustup-init.sh

ENV PATH=$PATH:/root/.cargo/bin
# ENV TRUNK_VERSION=v0.14.0

# RUN curl -sSf -L https://github.com/thedodd/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /root/.cargo/bin
# RUN cargo install wasm-bindgen-cli -j 8

RUN cargo install trunk wasm-bindgen-cli -j 8

RUN rustup toolchain list && env