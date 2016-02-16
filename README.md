includedir
===========

[![Build Status](https://img.shields.io/travis/tilpner/includedir.svg?style=flat-square)](https://travis-ci.org/tilpner/includedir)
[![Crates.io version](https://img.shields.io/crates/v/includedir.svg?style=flat-square)](https://crates.io/crates/includedir)
[![Crates.io license](https://img.shields.io/crates/l/includedir.svg?style=flat-square)](https://crates.io/crates/includedir)

Include a directory in your Rust binary, e.g. static files for your web server or assets for your game.

## Example

**Cargo.toml**
```toml
[package]
name = "example"
version = "0.1.0"

build = "build.rs"
include = ["data"]

[dependencies]
phf = "0.7.12"

[build-dependencies]
includedir = "0.1.1"
```

**build.rs**

```rust
extern crate includedir;

fn main() {
    includedir::build("data").unwrap();
}
```

**src/main.rs**

```rust
extern crate phf;

include!(concat!(env!("OUT_DIR"), "/dir_data.rs"));

fn main() {
    for (k, v) in FILES.entries() {
        println!("{}: {} bytes", k, v.len());
    }
}
```
