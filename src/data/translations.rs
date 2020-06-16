use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;
use zip::ZipArchive;

pub struct Translations {
    entries: HashMap<String, String>,
}

impl Translations {
    pub fn load(path: &Path) -> io::Result<Translations> {
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

    pub fn get(&self, translation: Translation, name: &str) -> Option<&String> {
        let key = format!("content.{}.{}", translation as i32, name);
        self.entries.get(&key)
    }
}

pub enum Translation {
    Spell = 3,
    Area = 6,
    Monster = 7,
    State = 8,
    StateDescription = 9,
    Effect = 10,
    ItemType = 14,
    Item = 15,
    ItemDescription = 16,
    Pet = 65,
    Instance = 77,
    InteractiveElementView = 99,
}
