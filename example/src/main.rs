extern crate includedir;
extern crate phf;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() {
    println!("{:?}", FILES.get("data/foo"))
    // for (k, v) in FILES.entries() {
    //    println!("{}: {} bytes", k, v.len());
    // }
}
