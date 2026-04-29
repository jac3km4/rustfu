use hashbrown::HashMap;

/// Represents the frame data of a sprite.
///
/// Based on the size of the data, the frames can be encoded in three formats:
/// bytes, shorts (16-bit), or ints (32-bit).
#[derive(Debug, Clone)]
pub enum FrameData {
    /// Frame data is encoded as an array of bytes. Used when maximum value is < 255.
    Bytes(Vec<u8>),
    /// Frame data is encoded as an array of 16-bit integers. Used when maximum value is < 65535.
    Shorts(Vec<u16>),
    /// Frame data is encoded as an array of 32-bit integers. Used for larger values.
    Ints(Vec<u32>),
}

/// Represents the payload of a sprite definition.
///
/// Sprite definitions can be encoded in several ways depending on the presence of
/// multiple frames, actions, etc.
#[derive(Debug, Clone)]
pub enum SpritePayload {
    /// Represents an indexed sprite with multiple frames.
    /// The payload contains: frame position array, frames duration, and action info array.
    Indexed(Vec<i32>, Vec<i16>, Vec<i16>),
    /// Represents a sprite with a single frame. Contains frames and frames duration.
    SingleFrame(Vec<i16>, Vec<i16>),
    /// Represents a single sprite. Contains the sprite id and an array.
    Single(i16, Vec<i16>),
    /// Represents a single sprite with no actions. Contains just the sprite id.
    SingleNoAction(i16),
}

/// Actions that can be triggered during an animation.
#[derive(Debug, Clone)]
pub enum Action {
    /// Add a particle system.
    /// Parameters: particle_id, offset_x, offset_y, offset_z.
    AddParticle(i32, Option<i16>, Option<i16>, Option<i16>),
    /// Delete the animated element.
    Delete,
    /// Stop the animation (end).
    End,
    /// Go to a specific animation name, optionally with a specific execution percentage.
    GoTo(String, Option<u8>),
    /// Go to an animation if a previous animation condition matches.
    GoToIfPrevious(Vec<String>, Vec<String>, Option<String>),
    /// Go to a random animation from a list, with corresponding percentage weights.
    GoToRandom(Vec<String>, Vec<u8>),
    /// Go to a static animation.
    GoToStatic,
    /// Trigger a hit action.
    Hit,
    /// Run a script.
    RunScript(String),
    /// Set the render radius.
    SetRadius(i8),
}

/// The root structure representing an Animation.
#[derive(Debug, Clone)]
pub struct Animation {
    /// The version flags of the animation.
    pub version: AnimationVersion,
    /// The frame rate of the animation.
    pub frame_rate: u8,
    /// Index data related to the animation, if any.
    pub index: Option<AnimationIndex>,
    /// The texture associated with the animation.
    pub texture: Option<Texture>,
    /// The shapes defined in the animation, indexed by their ID.
    pub shapes: HashMap<i16, Shape>,
    /// Optional transform table defining rotations, translations, etc.
    pub transform: Option<TransformTable>,
    /// The sprites making up the animation, indexed by their ID.
    pub sprites: HashMap<i16, Sprite>,
    /// Imported assets.
    pub imports: Vec<Import>,
}

impl Animation {
    /// Helper to get the scale of the animation, defaulting to 1.0 if not specified.
    #[inline]
    pub fn scale(&self) -> f32 {
        self.index.as_ref().and_then(|i| i.scale).unwrap_or(1.)
    }
}

/// A wrapper around a byte flag representing the animation version settings.
#[derive(Debug, Clone)]
pub struct AnimationVersion(pub u8);

impl AnimationVersion {
    /// Check if the animation uses an atlas (`0x1` flag).
    pub fn use_atlas(&self) -> bool {
        self.0 & 0x1 == 0x1
    }

    /// Check if the animation uses a local index (`0x2` flag).
    pub fn use_local_index(&self) -> bool {
        self.0 & 0x2 == 0x2
    }

    /// Check if the animation uses perfect hit test (`0x4` flag).
    pub fn use_perfect_hit_test(&self) -> bool {
        self.0 & 0x4 == 0x4
    }

    /// Check if the animation is optimized (`0x8` flag).
    pub fn is_optimized(&self) -> bool {
        self.0 & 0x8 == 0x8
    }

    /// Check if the animation uses a transform index (`0x10` flag).
    pub fn use_transform_index(&self) -> bool {
        self.0 & 0x10 == 0x10
    }
}

/// Represents the index of an animation, containing metadata like scale and hideable parts.
#[derive(Debug, Clone)]
pub struct AnimationIndex {
    /// The flags for the animation index.
    pub flags: AnimationFlags,
    /// Optional overall scale factor for the animation.
    pub scale: Option<f32>,
    /// Optional render radius used for occlusion/culling.
    pub render_radius: Option<f32>,
    /// Optional array of file names extensions.
    pub file_names: Option<Vec<String>>,
    /// List of file records for the animation.
    pub animation_files: Vec<AnimationFile>,
    /// Optional list of parts that are hidden by specific items.
    pub parts_to_be_hidden: Option<Vec<HiddenPart>>,
    /// Optional list of parts that can hide other items.
    pub parts_hidden_by: Option<Vec<HideablePart>>,
    /// Additional animation extension data.
    pub extension: Option<AnimationExtension>,
}

/// A wrapper around a byte flag defining features present in the Animation Index.
#[derive(Debug, Clone)]
pub struct AnimationFlags(pub u8);

impl AnimationFlags {
    /// Check if a scale is defined (`0x1` flag).
    pub fn has_scale(&self) -> bool {
        self.0 & 0x1 == 0x1
    }

    /// Check if extension file names are defined (`0x2` flag).
    pub fn has_extension(&self) -> bool {
        self.0 & 0x2 == 0x2
    }

    /// Check if parts hidden by items are defined (`0x4` flag).
    pub fn has_hiding_part(&self) -> bool {
        self.0 & 0x4 == 0x4
    }

    /// Check if a render radius is defined (`0x8` flag).
    pub fn has_render_radius(&self) -> bool {
        self.0 & 0x8 == 0x8
    }

    /// Check if the animation allows overriding the flip (`0x10` flag is 0).
    pub fn use_flip(&self) -> bool {
        self.0 & 0x10 == 0x10
    }

    /// Check if a perfect hit test is used (`0x20` flag).
    pub fn use_perfect_hit_test(&self) -> bool {
        self.0 & 0x20 == 0x20
    }

    /// Check if the animation has parts that can hide other items (`0x40` flag).
    pub fn can_hide_part(&self) -> bool {
        self.0 & 0x40 == 0x40
    }

    /// Check if the animation is extended with height and color data (`0x80` flag).
    pub fn is_extended(&self) -> bool {
        self.0 & 0x80 == 0x80
    }
}

/// Represents a record mapping an animation name to a file index.
#[derive(Debug, Clone)]
pub struct AnimationFile {
    /// Name of the animation file.
    pub name: String,
    /// CRC hash of the animation part name.
    pub crc: i32,
    /// Index referencing a specific file in the `file_names` array.
    pub file_index: i16,
}

/// Represents a part that can be hidden by equipping a specific item.
#[derive(Debug, Clone)]
pub struct HiddenPart {
    /// Name of the item.
    pub item_name: String,
    /// CRC key of the item.
    pub crc_key: i32,
}

/// Represents a part that hides another part when equipped.
#[derive(Debug, Clone)]
pub struct HideablePart {
    /// CRC key of the part.
    pub crc_key: i32,
    /// CRC key of the part to hide.
    pub crc_to_hide: i32,
}

/// Contains extended information like animation heights and highlight color.
#[derive(Debug, Clone)]
pub struct AnimationExtension {
    /// Optional heights associated with animation names (stored by their CRC/hash).
    pub heights: Option<HashMap<i32, i8>>,
    /// Optional highlight color override.
    pub highlight_color: Option<Color>,
}

/// Represents an imported asset.
#[derive(Debug, Clone)]
pub struct Import {
    /// Import ID.
    pub id: i16,
    /// Import Name.
    pub name: String,
    /// CRC hash of the import part name.
    pub crc: i32,
}

/// Represents a texture definition within an animation.
#[derive(Debug, Clone)]
pub struct Texture {
    /// Name of the texture.
    pub name: String,
    /// CRC of the texture.
    pub crc: i32,
}

/// Contains pre-computed arrays for transformations like colors, rotations, translations, and actions.
#[derive(Debug, Clone)]
pub struct TransformTable {
    /// Array of float values representing color transformations.
    pub colors: Vec<f32>,
    /// Array of float values representing rotation and skew transformations.
    pub rotations: Vec<f32>,
    /// Array of float values representing translation transformations.
    pub translations: Vec<f32>,
    /// Array of actions.
    pub actions: Vec<Action>,
}

impl TransformTable {
    /// Provides an empty transform table.
    pub const EMPTY: TransformTable = TransformTable {
        colors: vec![],
        rotations: vec![],
        translations: vec![],
        actions: vec![],
    };
}

/// Defines a 2D shape or sprite quad region within the texture atlas.
#[derive(Debug, Clone)]
pub struct Shape {
    /// Unique ID of the shape.
    pub id: i16,
    /// Index of the texture this shape belongs to.
    pub texture_index: i16,
    /// Top UV coordinate of the shape in the texture atlas.
    pub top: f32,
    /// Left UV coordinate of the shape in the texture atlas.
    pub left: f32,
    /// Bottom UV coordinate of the shape in the texture atlas.
    pub bottom: f32,
    /// Right UV coordinate of the shape in the texture atlas.
    pub right: f32,
    /// Width of the shape in pixels.
    pub width: u16,
    /// Height of the shape in pixels.
    pub height: u16,
    /// X offset for rendering the shape.
    pub offset_x: f32,
    /// Y offset for rendering the shape.
    pub offset_y: f32,
}

/// Represents a sprite definition, consisting of multiple frames, transformations, or nested sprites.
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Unique ID of the sprite.
    pub id: i16,
    /// Name info for the sprite, including CRC.
    pub name: SpriteName,
    /// Sprite definition flags.
    pub flags: SpriteFlags,
    /// The frame data points used to reconstruct the sprite sequence.
    pub frame_data: FrameData,
    /// The payload specifying if this sprite is single frame, indexed, etc.
    pub payload: SpritePayload,
}

impl Sprite {
    /// Calculates the number of frames contained in this sprite.
    pub fn frame_count(&self) -> usize {
        match &self.payload {
            SpritePayload::Indexed(frame_pos, _, action_info) => {
                let divisor = if action_info.is_empty() { 2 } else { 3 };
                frame_pos.len() / divisor
            }
            _ => 1,
        }
    }
}

/// A wrapper around a byte flag defining features for a Sprite.
#[derive(Debug, Clone)]
pub struct SpriteFlags(pub u8);

impl SpriteFlags {
    /// Checks if the sprite has a custom name defined (`0x40` flag).
    pub fn has_name(&self) -> bool {
        self.0 & 0x40 == 0x40
    }
}

/// Contains name and CRC properties for a sprite.
#[derive(Debug, Clone)]
pub struct SpriteName {
    /// Optional textual name of the sprite.
    pub name: Option<String>,
    /// Complete name CRC.
    pub name_crc: i32,
    /// Base name CRC (often omitting the prefix).
    pub base_name_crc: i32,
}

/// A standard RGBA color structure using floats.
#[derive(Debug, Clone)]
pub struct Color {
    /// Red channel value (0.0 to 1.0).
    pub red: f32,
    /// Green channel value (0.0 to 1.0).
    pub green: f32,
    /// Blue channel value (0.0 to 1.0).
    pub blue: f32,
    /// Alpha channel value (0.0 to 1.0).
    pub alpha: f32,
}

impl Color {
    /// A constant representing solid white color.
    pub const WHITE: Color = Color::new(1., 1., 1., 1.);

    /// Create a new color with RGBA float components.
    #[inline]
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl From<Color> for [f32; 4] {
    #[inline]
    fn from(color: Color) -> Self {
        [color.red, color.green, color.blue, color.alpha]
    }
}
