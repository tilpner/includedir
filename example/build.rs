extern crate includedir_codegen;

use std::env;

use includedir_codegen::Compression;

fn main() {
    includedir_codegen::start("FILES")
        .passthrough(env::var("PASSTHROUGH").is_ok())
        .dir("data", Compression::Gzip)
        .build("data.rs")
        .unwrap();
}
