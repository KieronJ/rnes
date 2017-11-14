use std::fs::File;
use std::path::Path;

pub fn open_file<P: AsRef<Path>>(path: P) -> File {
    File::open(path).unwrap()
}