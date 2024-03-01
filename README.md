# axum-aws-lambda

[![Rust](https://github.com/lazear/axum-aws-lambda/actions/workflows/rust.yml/badge.svg)](https://github.com/lazear/axum-aws-lambda/actions/workflows/rust.yml)
![crates.io](https://img.shields.io/crates/v/axum-aws-lambda)

This crate provides a `tower::Layer` that translates `hyper`/`axum` requests to the format used by the `aws-lambda-rust-runtime` crate. This allows users to switch between just running a Hyper server, and running under the Lambda runtime - this dramatically speeds up development! It also means that you can use off-the-shelf components from the Tower ecosystem!

Check out `examples/main.rs`: running in debug mode runs a hyper server, and building for release mode compiles using the Lambda runtime.

### Testing out the Lambda runtime locally

There is an example Dockerfile for locally spinning up a lambda runtime:

```terminal
cargo build --release --example main
docker build . -t lambda-test
docker run -p 9000:8080 lambda-test
```

In `test-lambda-runtime/` there is a python script for testing and a Dockerfile for running it. 

In another shell, from the root of this repository:

```terminal
cd test-lambda-runtime
docker build . -t test_lambda_runtime
docker run --network="host" test_lambda_runtime
```
