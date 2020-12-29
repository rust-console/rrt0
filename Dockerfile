FROM rust:latest
RUN mkdir -p /app
WORKDIR /app
COPY rust-toolchain ./
RUN rustup update && \
    rustup component add clippy && \
    rustup component add miri && \
    rustup component add rust-src && \
    rustup component add rustfmt
ENV RUSTFLAGS='-Z sanitizer=thread'
ENV RUSTDOCFLAGS='-Z sanitizer=thread'
COPY Cargo.* ./
COPY src/ ./src/
RUN set -x && \
    cargo check --all && \
    cargo fmt --all -- --check && \
    cargo clippy --all --tests -- -D warnings && \
    cargo miri test --all --all-features && \
    cargo test --all --all-features -Z build-std \
        --target $(basename $(dirname $(rustc --print target-libdir))) && \
    cargo doc --workspace --no-deps
