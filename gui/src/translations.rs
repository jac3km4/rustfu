use std::fs::File;
use std::io;
use std::path::Path;

use hashbrown::HashMap;
use zip::ZipArchive;

#[derive(Debug)]
pub struct Translations {
    entries: HashMap<String, String>,
}

impl Translations {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Translations> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        let input = io::BufReader::new(archive.by_index(0)?);
        Translations::read(input)
    }

    fn read<R: io::BufRead>(source: R) -> io::Result<Translations> {
        let mut entries = HashMap::new();
        for read in source.lines() {
            let line = read?;
            if let Some(idx) = line.find('=') {
                let (key, val) = line.split_at(idx);
                entries.insert(key.to_owned(), val[1..].to_owned());
            }
        }
        Ok(Translations { entries })
    }

    pub fn get(&self, translation_id: &str, name: &str) -> Option<&String> {
        let key = format!("content.{}.{}", translation_id, name);
        self.entries.get(&key)
    }
}
