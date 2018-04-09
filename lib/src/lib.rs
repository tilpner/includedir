extern crate phf;

#[cfg(feature = "flate2")]
extern crate flate2;

use std::borrow::{Borrow, Cow};
use std::io::{self, BufReader, Cursor, Error, ErrorKind, Read};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "flate2")]
use flate2::bufread::GzDecoder;

pub enum Compression {
    None,
    Gzip
}

/// Runtime access to the included files
pub struct Files {
    // Do not access these fields, they are only public to allow for code generation!
    #[doc(hidden)]
    pub files: phf::Map<&'static str, (Compression, &'static [u8])>,
    #[doc(hidden)]
    pub passthrough: AtomicBool
}

#[cfg(windows)]
fn as_key(path: &str) -> Cow<str> {
    Cow::Owned(path.replace("\\", "/"))
}

#[cfg(not(windows))]
fn as_key(path: &str) -> Cow<str> {
    Cow::Borrowed(path)
}

impl Files {
    pub fn set_passthrough(&self, enabled: bool) {
        self.passthrough.store(enabled, Ordering::Relaxed);
    }

    pub fn is_available(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    /// Returns an iterator over all available file names.  Does not
    /// decompress any compressed data.
    pub fn file_names(&'static self) -> FileNames {
        FileNames { iter: self.files.keys() }
    }

    pub fn get(&self, path: &str) -> io::Result<Cow<'static, [u8]>> {
        if self.passthrough.load(Ordering::Relaxed) {
            let mut r = BufReader::new(File::open(path)?);
            let mut v = Vec::new();
            r.read_to_end(&mut v)?;
            return Ok(Cow::Owned(v))
        }

        let key = as_key(path);
        match self.files.get(key.borrow() as &str) {
            Some(b) => {
                match b.0 {
                    Compression::None => Ok(Cow::Borrowed(b.1)),
                    #[cfg(feature = "flate2")]
                    Compression::Gzip => {
                        let mut r = GzDecoder::new(Cursor::new(b.1));
                        let mut v = Vec::new();
                        r.read_to_end(&mut v)?;
                        Ok(Cow::Owned(v))
                    }
                    #[cfg(not(feature = "flate2"))]
                    Compression::Gzip => panic!("Feature 'flate2' not enabled"),
                }
            }
            None => Err(Error::new(ErrorKind::NotFound, "Key not found")),
        }
    }

    pub fn read(&self, path: &str) -> io::Result<Box<Read>> {
        if self.passthrough.load(Ordering::Relaxed) {
            return Ok(Box::new(BufReader::new(File::open(path)?)))
        }

        let key = as_key(path);
        match self.files.get(key.borrow() as &str) {
            Some(b) => {
                match b.0 {
                    Compression::None => Ok(Box::new(Cursor::new(b.1))),
                    #[cfg(feature = "flate2")]
                    Compression::Gzip => Ok(Box::new(GzDecoder::new(Cursor::new(b.1)))),
                    #[cfg(not(feature = "flate2"))]
                    Compression::Gzip => panic!("Feature 'flate2' not enabled"),
                }
            }
            None => Err(Error::new(ErrorKind::NotFound, "Key not found")),
        }
    }
}

/// Iterates over the file names available for `Files` object.
pub struct FileNames {
    // Our internal iterator.  We wrap this in a nice struct so our
    // caller doesn't need to know the details.
    iter: phf::map::Keys<'static, &'static str, (Compression, &'static [u8])>,
}

impl Iterator for FileNames {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|s| *s)
    }
}
