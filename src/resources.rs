use std::fs::File;
use std::io;
use std::io::{Cursor, Read};
use std::path::Path;

use zip::ZipArchive;

use crate::decode::*;
use crate::types::Animation;

pub struct Resources<R: io::Read + io::Seek> {
    pub npcs: AnimationArchive<R>,
    pub players: AnimationArchive<R>,
    pub interactives: AnimationArchive<R>,
    pub pets: AnimationArchive<R>,
}

impl Resources<File> {
    pub fn open(path: &Path) -> io::Result<Resources<File>> {
        let root = path.join("animations");
        let npcs = AnimationArchive::open(&root.join("npcs").join("npcs.jar"))?;
        let players = AnimationArchive::open(&root.join("players").join("players.jar"))?;
        let interactives = AnimationArchive::open(&root.join("interactives").join("interactives.jar"))?;
        let pets = AnimationArchive::open(&root.join("pets").join("pets.jar"))?;
        Ok(Resources {
            npcs,
            players,
            interactives,
            pets,
        })
    }
}

pub struct AnimationArchive<R: io::Read + io::Seek> {
    archive: ZipArchive<R>,
}

impl AnimationArchive<File> {
    pub fn open(path: &Path) -> io::Result<AnimationArchive<File>> {
        let archive = ZipArchive::new(File::open(path)?).unwrap();
        Ok(AnimationArchive { archive })
    }

    pub fn load_animation(&mut self, id: &str) -> io::Result<Animation> {
        let mut entry = self.archive.by_name(&format!("{}.anm", id))?;
        entry.decode()
    }

    pub fn load_texture(&mut self, id: &str) -> io::Result<image::RgbaImage> {
        let mut entry = self.archive.by_name(&format!("Atlas/{}.png", id))?;
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        let image = image::load(Cursor::new(buf), image::ImageFormat::Png)
            .unwrap()
            .to_rgba();
        Ok(image)
    }

    pub fn list_animations(&self) -> impl Iterator<Item = &str> {
        self.archive
            .file_names()
            .filter(|e| e.ends_with(".anm"))
            .map(|e| e.trim_end_matches(".anm"))
    }
}
