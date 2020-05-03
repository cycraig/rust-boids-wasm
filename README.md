# Rust-Boids-Wasm

A simple, unoptimised implementation of [boids](https://cs.stanford.edu/people/eroberts/courses/soco/projects/2008-09/modeling-natural-systems/boids.html) in [Rust](https://www.rust-lang.org/) targeting [WebAssembly](https://webassembly.org/).

### Prerequisites

-To compile this project locally, you need to [install Rust](https://www.rust-lang.org/tools/install) then run the following to ensure you can compile to the wasm target:

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

To run the unit tests:

```
cargo make test
```

### Release

To compile an optimised, stripped-down release version to `./pkg`:

```
cargo make build_release
```

N.B. make sure to include the packaged files explicitly:

```
git add ./pkg/boids.js ./pkg/boids_bg.wasm -f
```
