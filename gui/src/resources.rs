use std::fs::File;
use std::io;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use rustfu_renderer::types::Animation;
use wakfudecrypt::document::Document;
use wakfudecrypt::types::interactive_element_model::InteractiveElementModel;
use wakfudecrypt::types::monster::Monster;
use wakfudecrypt::types::pet::Pet;
use wakfudecrypt::BinaryData;
use zip::ZipArchive;

use crate::translations::Translations;

#[derive(Debug)]
pub struct Resources<R: io::Read + io::Seek> {
    root: PathBuf,

    pub npc_animations: AnimationArchive<R>,
    pub interactive_animations: AnimationArchive<R>,
    pub pet_animations: AnimationArchive<R>,

    pub translations: Translations,
}

impl Resources<File> {
    pub fn open(root: impl AsRef<Path>) -> io::Result<Resources<File>> {
        let anim_root = root.as_ref().join("contents").join("animations");
        let npcs = AnimationArchive::open(anim_root.join("npcs").join("npcs.jar"))?;
        let interactives =
            AnimationArchive::open(anim_root.join("interactives").join("interactives.jar"))?;
        let pets = AnimationArchive::open(anim_root.join("pets").join("pets.jar"))?;

        let translations = Translations::load(
            root.as_ref()
                .join("contents")
                .join("i18n")
                .join("i18n_en.jar"),
        )?;

        Ok(Resources {
            root: root.as_ref().to_owned(),
            npc_animations: npcs,
            interactive_animations: interactives,
            pet_animations: pets,
            translations,
        })
    }
}

impl<R: io::Read + io::Seek> Resources<R> {
    pub fn load_data<A: BinaryData>(&mut self) -> io::Result<Document<A>> {
        Document::load(&self.root)
    }
}

#[derive(Debug)]
pub struct AnimationArchive<R> {
    archive: ZipArchive<R>,
}

impl AnimationArchive<File> {
    pub fn open(path: impl AsRef<Path>) -> io::Result<AnimationArchive<File>> {
        let file = File::open(&path)?;
        let archive = ZipArchive::new(file)?;
        Ok(AnimationArchive { archive })
    }

    pub fn load_animation(&mut self, id: &str) -> io::Result<Animation> {
        let mut entry = self.archive.by_name(&format!("{}.anm", id))?;
        rustfu_renderer::decode::Decode::decode(&mut entry)
    }

    pub fn load_texture(&mut self, id: &str) -> anyhow::Result<image::RgbaImage> {
        let mut entry = self.archive.by_name(&format!("Atlas/{}.png", id))?;
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        let image = image::load(Cursor::new(buf), image::ImageFormat::Png)?.to_rgba8();
        Ok(image)
    }
}

pub trait AnimatedEntity {
    const TRANSLATION_ID: &'static str;

    fn id(&self) -> i32;
    fn gfx(&self) -> i32;
}

impl AnimatedEntity for Monster {
    const TRANSLATION_ID: &'static str = "7";

    #[inline]
    fn id(&self) -> i32 {
        self.id
    }

    #[inline]
    fn gfx(&self) -> i32 {
        self.gfx
    }
}

impl AnimatedEntity for InteractiveElementModel {
    const TRANSLATION_ID: &'static str = "99";

    #[inline]
    fn id(&self) -> i32 {
        self.id
    }

    #[inline]
    fn gfx(&self) -> i32 {
        self.gfx
    }
}

impl AnimatedEntity for Pet {
    const TRANSLATION_ID: &'static str = "15";

    #[inline]
    #[allow(clippy::misnamed_getters)]
    fn id(&self) -> i32 {
        self.item_ref_id
    }

    #[inline]
    fn gfx(&self) -> i32 {
        self.gfx_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatedEntityKind {
    Monster,
    InteractiveElementModel,
    Pet,
}

impl AnimatedEntityKind {
    pub fn label(self) -> &'static str {
        match self {
            AnimatedEntityKind::Monster => "Monsters",
            AnimatedEntityKind::InteractiveElementModel => "Interactives",
            AnimatedEntityKind::Pet => "Pets",
        }
    }
}

#[derive(Debug)]
pub struct AnimationEntry {
    label: String,
    id: i32,
}

impl AnimationEntry {
    pub fn load_all<R, A>(resources: &mut Resources<R>) -> io::Result<Vec<Self>>
    where
        R: io::Read + io::Seek,
        A: BinaryData + AnimatedEntity,
    {
        let result = resources
            .load_data::<A>()?
            .elements
            .iter()
            .map(|elem| {
                let label = resources
                    .translations
                    .get(A::TRANSLATION_ID, &elem.id().to_string());
                AnimationEntry {
                    label: label
                        .cloned()
                        .unwrap_or_else(|| format!("Unnamed {}", elem.id())),
                    id: elem.gfx(),
                }
            })
            .collect();
        Ok(result)
    }

    #[inline]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[inline]
    pub fn id(&self) -> i32 {
        self.id
    }
}
