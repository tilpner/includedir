extern crate walkdir;
extern crate phf_codegen;

use std::{env, io};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use walkdir::WalkDir;

pub fn build<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    let dir = dir.as_ref();

    let base_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_owned();

    let dir_name = dir.file_name().expect("Invalid directory name").to_string_lossy();
    let out_name = format!("dir_{}.rs", dir_name);
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join(out_name);
    let mut out_file = BufWriter::new(File::create(&out_path).unwrap());

    write!(&mut out_file,
           "static FILES: phf::Map<&'static str, &'static [u8]> = ")
        .unwrap();

    // FIXME: rustfmt mangles this
    let entries: Vec<(String, String)> = WalkDir::new(dir)
                                             .into_iter()
                                             .filter(|e| {
                                                 !e.as_ref()
                                                   .ok()
                                                   .map_or(true, |e| e.file_type().is_dir())
                                             })
                                             .map(|e| {
                                                 let entry = e.expect("Invalid dir entry.");
                                                 let name = format!("{}", entry.path().display());
                                                 let code = format!("include_bytes!(\"{}\") \
                                                                     as &'static [u8]",
                                                                    base_path.join(entry.path())
                                                                             .display());
                                                 (name, code)
                                             })
                                             .collect();

    let mut map: phf_codegen::Map<&str> = phf_codegen::Map::new();
    for &(ref name, ref code) in &entries {
        map.entry(name, code);
    }
    map.build(&mut out_file).unwrap();

    write!(&mut out_file, ";\n").unwrap();
    Ok(())
}
