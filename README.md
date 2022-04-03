[![License: BSD 2-Clause](https://img.shields.io/badge/License-BSD%202--Clause-blue)](LICENSE)
## Description
Converts binary image data from the game "Out of the World" into PNG files.

## Compiling/Building
If you wish to build from source, for your own system, Rust is integrated with the `cargo` build system. To install Rust and `cargo`, just follow [these instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html). Once installed, while in the project's root directory, run `cargo build --release` to build, or use `cargo run --release -- input_file_name.bin` to run directly. The built binary will be available in `./target/release/`

To cross-compile builds for other operating systems, you can use [rust-embedded/cross](https://github.com/rust-embedded/cross).