[![Build status](https://badge.buildkite.com/c23f47f4a827f04daece909963bd3a248496f0cdbabfbecee4.svg?branch=master)](https://buildkite.com/temporal/core-sdk?branch=master)

Core SDK that can be used as a base for all other Temporal SDKs.

# Getting started

See the [Architecture](ARCHITECTURE.md) doc for some high-level information.

This repo uses a subtree for upstream protobuf files. The path `protos/api_upstream` is a 
subtree. To update it, use:
`git subtree pull --prefix protos/api_upstream/ git://github.com/temporalio/api.git master --squash`

## Dependencies
* Protobuf compiler

# Development

All of the following commands are enforced for each pull request.

## Building and testing

You can buld and test the project using cargo:
`cargo build`
`cargo test`

Run integ tests with `cargo integ-test`. You will need to already be running the server:
`docker-compose -f .buildkite/docker/docker-compose.yaml up`

## Formatting
To format all code run:
`cargo fmt --all`

## Linting
We are using [clippy](https://github.com/rust-lang/rust-clippy) for linting.
You can run it using:
`cargo clippy --all -- -D warnings`

## Debugging
The crate uses [tracing](https://github.com/tokio-rs/tracing) to help with debugging. To enable
it for a test, insert the below snippet at the start of the test. By default, tracing data is output
to stdout in a (reasonably) pretty manner.

```rust
crate::telemetry::telemetry_init(Default::default());
let s = info_span!("Test start");
let _enter = s.enter();
```

The passed in options to initialization can be customized to export to an OTel collector, etc.

To run integ tests with OTel collection on, you can use `integ-with-otel.sh`. You will want to make
sure you are running the collector via docker, which can be done like so:
`docker-compose -f .buildkite/docker/docker-compose.yaml -f .buildkite/docker/docker-compose-telem.yaml up`


If you are working on a language SDK, you are expected to initialize tracing early in your `main`
equivalent.

## Style Guidelines

### Error handling
Any error which is returned from a public interface should be well-typed, and we use 
[thiserror](https://github.com/dtolnay/thiserror) for that purpose.

Errors returned from things only used in testing are free to use 
[anyhow](https://github.com/dtolnay/anyhow) for less verbosity.

