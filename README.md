# Turing-Machine

Turing Machine backed by a simple stack machine. This stack machine is entirely based on [rocketlang](https://github.com/artificialnull/rocketlang), by artificialnull.

### Setting up Rust environment

You don't need to do this if you're just running the server in docker-compose,
but it's nice to have for development and allows for using a debugger.

First, install rustup (Google it). Then:

```
rustup component add clippy-preview rustfmt-preview rust-analysis rust-src rls
```
