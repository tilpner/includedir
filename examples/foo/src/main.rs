extern crate phf;

include!(concat!(env!("OUT_DIR"), "/dir_data.rs"));

fn main() {
    for (k, v) in FILES.entries() {
        println!("{}: {} bytes", k, v.len());
    }
}
