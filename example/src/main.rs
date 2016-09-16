extern crate includedir;
extern crate phf;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() {
    println!("{:?}", FILES.get("data/foo"));

    for name in FILES.file_names() {
        println!("Found: {}", name);
    }
}

#[test]
fn test() {
    assert_eq!(FILES.get("data/foo").expect("data/foo not present").into_owned(), &[102, 111, 111, 10]);

    let files = FILES.file_names().collect::<Vec<_>>();
    assert_eq!(files, vec!["data/inner/boom", "data/foo", "data/empty"]);

    assert_eq!(FILES.get("data/inner/boom").unwrap().into_owned(), &[98, 111, 111, 109, 10]);
    assert_eq!(FILES.get("data/foo").unwrap().into_owned(), &[102, 111, 111, 10]);
    assert_eq!(FILES.get("data/empty").unwrap().into_owned(), &[]);
}
