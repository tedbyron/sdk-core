FROM rust:1.56

RUN rustup component add rustfmt && \
	rustup component add clippy

RUN cargo install cargo-tarpaulin

WORKDIR /sdk-core
