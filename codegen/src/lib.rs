extern crate walkdir;
extern crate phf_codegen;
extern crate flate2;

use std::{env, fmt, io};
use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use flate2::FlateWriteExt;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Compression {
    None,
    Gzip,
    Passthrough,
}

impl fmt::Display for Compression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Compression::None => fmt.write_str("None"),
            Compression::Gzip => fmt.write_str("Gzip"),
            Compression::Passthrough => panic!("Should not be called"),
        }
    }
}

pub struct IncludeDir {
    files: HashMap<String, (Compression, PathBuf)>,
    name: String,
    passthrough: bool,
}

pub fn start(static_name: &str) -> IncludeDir {
    IncludeDir {
        files: HashMap::new(),
        name: static_name.to_owned(),
        passthrough: false,
    }
}

#[cfg(windows)]
fn as_key(path: &str) -> Cow<str> {
    Cow::Owned(path.replace("\\", "/"))
}

#[cfg(not(windows))]
fn as_key(path: &str) -> Cow<str> {
    Cow::Borrowed(path)
}

impl IncludeDir {
    /// Don't include any data, but read from the source directory instead.
    pub fn passthrough(&mut self) -> &mut IncludeDir {
        self.passthrough = true;
        self
    }

    /// Add a single file to the binary.
    /// With Gzip compression, the file will be encoded to OUT_DIR first.
    /// For chaining, it's not sensible to return a Result. If any to-be-included
    /// files can't be found, or encoded, this function will panic!.
    pub fn file<P: AsRef<Path>>(&mut self, path: P, comp: Compression) -> &mut IncludeDir {
        self.add_file(path, comp).unwrap();
        self
    }

    /// ## Panics
    ///
    /// This function panics when CARGO_MANIFEST_DIR or OUT_DIR are not defined.
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P, comp: Compression) -> io::Result<()> {
        let key = path.as_ref().to_string_lossy();

        match comp {
            c if self.passthrough || c == Compression::Passthrough => {
                self.files.insert(as_key(key.borrow()).into_owned(),
                                  (Compression::Passthrough, PathBuf::new()));

            }
            Compression::None => {
                self.files.insert(as_key(key.borrow()).into_owned(),
                                  (comp, path.as_ref().clone().to_owned()));
            }
            Compression::Gzip => {
                // gzip encode file to OUT_DIR
                let in_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(&path);
                let mut in_file = BufReader::new(try!(File::open(&in_path)));

                let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join(&path);
                try!(fs::create_dir_all(&out_path.parent().unwrap()));
                let out_file = BufWriter::new(try!(File::create(&out_path)));
                let mut encoder = out_file.gz_encode(flate2::Compression::Default);

                try!(io::copy(&mut in_file, &mut encoder));

                self.files.insert(as_key(key.borrow()).into_owned(),
                                  (comp, out_path.to_owned()));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Add a whole directory recursively to the binary.
    /// This function calls `file`, and therefore will panic! on missing files.
    pub fn dir<P: AsRef<Path>>(&mut self, path: P, comp: Compression) -> &mut IncludeDir {
        self.add_dir(path, comp).unwrap();
        self
    }

    /// ## Panics
    ///
    /// This function panics when CARGO_MANIFEST_DIR or OUT_DIR are not defined.
    pub fn add_dir<P: AsRef<Path>>(&mut self, path: P, comp: Compression) -> io::Result<()> {
        for entry in WalkDir::new(path).follow_links(true).into_iter() {
            match entry {
                Ok(ref e) if !e.file_type().is_dir() => {
                    try!(self.add_file(e.path(), comp));
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn build(&self, out_name: &str) -> io::Result<()> {
        let base_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_owned();
        let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join(out_name);
        let mut out_file = BufWriter::new(try!(File::create(&out_path)));

        try!(write!(&mut out_file,
                    "use includedir::*;\n\
                    pub static {}: Files = Files {{\n\
                    \tfiles:  ",
                    self.name));

        let entries: Vec<_> = self.files
                                  .iter()
                                  .map(|(name, &(ref comp, ref path))| {
                                      let include_path = format!("{}",
                                                                 base_path.join(path).display());
                                      if comp == &Compression::Passthrough {
                                          (as_key(&name).into_owned(),
                                           "(Compression::Passthrough, &[] as &'static [u8])"
                                               .to_owned())
                                      } else {
                                          let code = format!("(Compression::{}, \
                                                              include_bytes!(\"{}\") as &'static \
                                                              [u8])",
                                                             comp,
                                                             as_key(&include_path));
                                          (as_key(&name).into_owned(), code)
                                      }
                                  })
                                  .collect();
        let mut map: phf_codegen::Map<&str> = phf_codegen::Map::new();
        for &(ref name, ref code) in &entries {
            map.entry(&name, &code);
        }
        try!(map.build(&mut out_file));

        try!(write!(&mut out_file, "\n}};\n"));
        Ok(())
    }
}
