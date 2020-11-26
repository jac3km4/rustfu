use crate::translations::Translations;
use rustfu_renderer::types::Animation;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use wakfudecrypt::document::Document;
use wakfudecrypt::BinaryData;
use zip::ZipArchive;

pub struct Resources<R: io::Read + io::Seek> {
    root: PathBuf,

    pub npc_animations: AnimationArchive<R>,
    pub interactive_animations: AnimationArchive<R>,
    pub pet_animations: AnimationArchive<R>,

    pub translations: Translations,
}

impl Resources<File> {
    pub fn open(root: &Path) -> io::Result<Resources<File>> {
        let anim_root = root.join("contents").join("animations");
        let npcs = AnimationArchive::open(&anim_root.join("npcs").join("npcs.jar"))?;
        let interactives = AnimationArchive::open(&anim_root.join("interactives").join("interactives.jar"))?;
        let pets = AnimationArchive::open(&anim_root.join("pets").join("pets.jar"))?;

        let translations = Translations::load(&root.join("contents").join("i18n").join("i18n_en.jar"))?;

        Ok(Resources {
            root: root.to_owned(),
            npc_animations: npcs,
            interactive_animations: interactives,
            pet_animations: pets,
            translations,
        })
    }

    pub fn load_data<A: BinaryData>(&mut self) -> io::Result<Document<A>> {
        Document::load(&self.root)
    }
}

pub struct AnimationArchive<R: io::Read + io::Seek> {
    archive: ZipArchive<R>,
}

impl AnimationArchive<File> {
    pub fn open(path: &Path) -> io::Result<AnimationArchive<File>> {
        let file =
            File::open(path).map_err(|err| io::Error::new(err.kind(), format!("{} for file {:?}", err, path)))?;
        let archive = ZipArchive::new(file)?;
        Ok(AnimationArchive { archive })
    }

    pub fn load_animation(&mut self, id: &str) -> io::Result<Animation> {
        let mut entry = self.archive.by_name(&format!("{}.anm", id))?;
        rustfu_renderer::decode::Decode::decode(&mut entry)
    }

    pub fn load_texture(&mut self, id: &str) -> io::Result<image::RgbaImage> {
        let mut entry = self.archive.by_name(&format!("Atlas/{}.png", id))?;
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        let image = image::load(Cursor::new(buf), image::ImageFormat::Png)
            .unwrap()
            .to_rgba8();
        Ok(image)
    }
}
