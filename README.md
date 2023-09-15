# Web renderer on Rust (Example)

This repo contains a functioning example of a small wgpu based web renderer based on the [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) tutorial.
The proyect is built entierly on Rust as a learning experience. The server is made with [axum](https://github.com/tokio-rs/axum) to serve files and [askama](https://github.com/djc/askama) for html templating (not really used at the time of writing this).

## How to run

You will need [wasm-pack](https://rustwasm.github.io/wasm-pack/) and [cargo](https://github.com/rust-lang/cargo).

- To run the renderer as a standalone program use:
```bash
cargo run --bin renderer
```

- To run the web renderer use:
```bash
make run
```