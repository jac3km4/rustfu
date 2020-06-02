use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result, Write};
use std::path::PathBuf;

pub struct Location(pub PathBuf);

impl Location {
    pub fn read(file: &File) -> Result<Location> {
        let line = BufReader::new(file)
            .lines()
            .next()
            .unwrap_or(Err(Error::new(ErrorKind::UnexpectedEof, "Expected one line")))?;
        Ok(Location(PathBuf::from(line)))
    }

    pub fn write(&self, file: &mut File) -> Result<()> {
        let str = self
            .0
            .as_path()
            .to_str()
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid path"))?;
        file.write(str.as_bytes())?;
        Ok(())
    }
}
