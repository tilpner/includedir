extern crate includedir;
extern crate phf;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() {
    println!("{:?}", FILES.get("data/foo"));

    for name in FILES.file_names() {
        println!("Found: {}", name);
    }
}
