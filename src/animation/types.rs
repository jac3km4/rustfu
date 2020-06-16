use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum FrameData {
    Bytes(Vec<u8>),
    Shorts(Vec<u16>),
    Ints(Vec<u32>),
}

#[derive(Debug, Clone)]
pub enum SpritePayload {
    Indexed(Vec<i32>, Vec<i16>, Vec<i16>),
    SingleFrame(Vec<i16>, Vec<i16>),
    Single(i16, Vec<i16>),
    SingleNoAction(i16),
}

#[derive(Debug, Clone)]
pub enum Action {
    AddParticle(i32, Option<i16>, Option<i16>, Option<i16>),
    Delete,
    End,
    GoTo(String, Option<u8>),
    GoToIfPrevious(Vec<String>, Vec<String>, Option<String>),
    GoToRandom(Vec<String>, Vec<u8>),
    GoToStatic,
    Hit,
    RunScript(String),
    SetRadius(i8),
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub version: AnimationVersion,
    pub frame_rate: u8,
    pub index: Option<AnimationIndex>,
    pub texture: Option<Texture>,
    pub shapes: HashMap<i16, Shape>,
    pub transform: Option<TransformTable>,
    pub sprites: HashMap<i16, Sprite>,
    pub imports: Vec<Import>,
}

#[derive(Debug, Clone)]
pub struct AnimationVersion(pub u8);

impl AnimationVersion {
    pub fn use_atlas(&self) -> bool {
        self.0 & 0x1 == 0x1
    }

    pub fn use_local_index(&self) -> bool {
        self.0 & 0x2 == 0x2
    }

    pub fn use_perfect_hit_test(&self) -> bool {
        self.0 & 0x4 == 0x4
    }

    pub fn is_optimized(&self) -> bool {
        self.0 & 0x8 == 0x8
    }

    pub fn use_transform_index(&self) -> bool {
        self.0 & 0x10 == 0x10
    }
}

#[derive(Debug, Clone)]
pub struct AnimationIndex {
    pub flags: AnimationFlags,
    pub scale: Option<f32>,
    pub render_radius: Option<f32>,
    pub file_names: Option<Vec<String>>,
    pub animation_files: Vec<AnimationFile>,
    pub parts_to_be_hidden: Option<Vec<HiddenPart>>,
    pub parts_hidden_by: Option<Vec<HideablePart>>,
    pub extension: Option<AnimationExtension>,
}

#[derive(Debug, Clone)]
pub struct AnimationFlags(pub u8);

impl AnimationFlags {
    pub fn has_scale(&self) -> bool {
        self.0 & 0x1 == 0x1
    }

    pub fn has_extension(&self) -> bool {
        self.0 & 0x2 == 0x2
    }

    pub fn has_hiding_part(&self) -> bool {
        self.0 & 0x4 == 0x4
    }

    pub fn has_render_radius(&self) -> bool {
        self.0 & 0x8 == 0x8
    }

    pub fn use_flip(&self) -> bool {
        self.0 & 0x10 == 0x10
    }

    pub fn use_perfect_hit_test(&self) -> bool {
        self.0 & 0x20 == 0x20
    }

    pub fn can_hide_part(&self) -> bool {
        self.0 & 0x40 == 0x40
    }

    pub fn is_extended(&self) -> bool {
        self.0 & 0x80 == 0x80
    }
}

#[derive(Debug, Clone)]
pub struct AnimationFile {
    pub name: String,
    pub crc: i32,
    pub file_index: i16,
}

#[derive(Debug, Clone)]
pub struct HiddenPart {
    pub item_name: String,
    pub crc_key: i32,
}

#[derive(Debug, Clone)]
pub struct HideablePart {
    pub crc_key: i32,
    pub crc_to_hide: i32,
}

#[derive(Debug, Clone)]
pub struct AnimationExtension {
    pub heights: Option<HashMap<i32, i8>>,
    pub highlight_color: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub id: i16,
    pub name: String,
    pub crc: i32,
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub name: String,
    pub crc: i32,
}

#[derive(Debug, Clone)]
pub struct TransformTable {
    pub colors: Vec<f32>,
    pub rotations: Vec<f32>,
    pub translations: Vec<f32>,
    pub actions: Vec<Action>,
}

impl TransformTable {
    pub const EMPTY: TransformTable = TransformTable {
        colors: vec![],
        rotations: vec![],
        translations: vec![],
        actions: vec![],
    };
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub id: i16,
    pub texture_index: i16,
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub width: u16,
    pub height: u16,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub id: i16,
    pub name: SpriteName,
    pub flags: SpriteFlags,
    pub frame_data: FrameData,
    pub payload: SpritePayload,
}

impl Sprite {
    pub fn frame_count(&self) -> usize {
        match &self.payload {
            SpritePayload::Indexed(frame_pos, _, action_info) => {
                let divisor = if action_info.len() == 0 { 2 } else { 3 };
                frame_pos.len() / divisor
            }
            _ => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpriteFlags(pub u8);

impl SpriteFlags {
    pub fn has_name(&self) -> bool {
        self.0 & 0x40 == 0x40
    }
}

#[derive(Debug, Clone)]
pub struct SpriteName {
    pub name: Option<String>,
    pub name_crc: i32,
    pub base_name_crc: i32,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}
