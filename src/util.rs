use std::fs::File;
use std::io;
use std::path::Path;

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<File, io::Error> {
    File::open(path)
}