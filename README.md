# Rust-Boids-Wasm

A simple, unoptimised implementation of [boids](https://cs.stanford.edu/people/eroberts/courses/soco/projects/2008-09/modeling-natural-systems/boids.html) in [Rust](https://www.rust-lang.org/) targeting [WebAssembly](https://webassembly.org/).

### Prerequisites

To compile this project locally, you need to [install Rust](https://www.rust-lang.org/tools/install) then run the following to ensure you can compile to the wasm target:

```
rustup update
rustup target add wasm32-unknown-unknown
```

Install cargo-make:

```
cargo install cargo-make
```

### Development

Build with debug symbols and start a local web server on port 8000:

```
cargo make build
cargo make serve
```

Alternatively, you can run `cargo make watch` to automatically recompile when files change.

#### Tests

To run the unit tests:

```
cargo make test
```

#### Benchmarks

To run the performance benchmark:

```
cargo bench
```

If you get the following error when trying to run the benchmarks
```
error[E0432]: unresolved import `boids`
 --> benches\bench_boids.rs:2:5
  |
2 | use boids::BoidFlock;
  |     ^^^^^ use of undeclared type or module `boids`
```

ensure that "rlib" is included in the `crate-type` specified in Cargo.toml, as [criterion.rs](https://github.com/bheisler/criterion.rs) does not support "cdylib". 

Note: specifying both "cdylib" and "rlib" in `crate-type` disables link-time-optimisations (lto). You can specify "cdylib" alone when building a release package, and "rlib" and "cdylib" during development. See these links for more information on the issue:

- https://users.rust-lang.org/t/cdylib-and-rlib-together-in-crate-type/6259
- https://github.com/rust-lang/cargo/issues/2301
- https://github.com/rust-lang/cargo/issues/4611

### Release

To compile an optimised, stripped-down release version to `./pkg`:

```
cargo make build_release
```

Manually minify `boids.js` -> `boids.min.js`.
