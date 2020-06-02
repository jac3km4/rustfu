use std::collections::HashMap;
use std::io;

use byteorder::*;

use crate::types::*;

pub trait Decode
where
    Self: Sized,
{
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self>;
}

pub trait DecodeExt
where
    Self: io::Read + Sized,
{
    fn decode<A: Decode>(&mut self) -> io::Result<A> {
        Decode::decode(self)
    }

    fn decode_prefixed<P: Decode + Into<u32>, A: Decode>(&mut self) -> io::Result<Vec<A>> {
        let count = self.decode::<P>()?;
        self.decode_n(count.into() as usize)
    }

    fn decode_n<A: Decode>(&mut self, count: usize) -> io::Result<Vec<A>> {
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(self.decode()?);
        }
        Ok(vec)
    }

    fn decode_opt<A: Decode>(&mut self, present: bool) -> io::Result<Option<A>> {
        if present {
            Ok(Some(self.decode()?))
        } else {
            Ok(None)
        }
    }
}

impl<R: io::Read> DecodeExt for R {}

impl<A: Decode, B: Decode> Decode for (A, B) {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        Ok((cursor.decode()?, cursor.decode()?))
    }
}

impl Decode for i8 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_i8()
    }
}

impl Decode for u8 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_u8()
    }
}

impl Decode for i16 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_i16::<LittleEndian>()
    }
}

impl Decode for u16 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_u16::<LittleEndian>()
    }
}

impl Decode for i32 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_i32::<LittleEndian>()
    }
}

impl Decode for u32 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_u32::<LittleEndian>()
    }
}

impl Decode for f32 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_f32::<LittleEndian>()
    }
}

impl Decode for f64 {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        cursor.read_f64::<LittleEndian>()
    }
}

impl Decode for String {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let mut buf = Vec::new();
        let mut c = cursor.read_u8()?;
        while c != 0 {
            buf.push(c);
            c = cursor.read_u8()?;
        }
        String::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
    }
}

impl Decode for Animation {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let version = cursor.decode::<AnimationVersion>()?;
        cursor.decode::<i16>()?;
        let frame_rate = cursor.decode::<u8>()?;
        let index = cursor.decode_opt::<AnimationIndex>(version.use_local_index())?;
        let texture_count = cursor.decode::<u16>()?;
        let texture = cursor.decode_opt::<Texture>(texture_count == 1)?;
        let shapes = cursor
            .decode_prefixed::<u16, Shape>()?
            .iter()
            .map(move |shape| (shape.id, shape.clone()))
            .collect();
        let transform = cursor.decode_opt::<TransformTable>(version.use_transform_index())?;
        let sprites_vec = cursor.decode_prefixed::<u16, Sprite>()?;
        let sprites = sprites_vec
            .iter()
            .map(move |sprite| (sprite.id, sprite.clone()))
            .collect();
        let imports = cursor.decode_prefixed::<u16, Import>()?;
        Ok(Animation {
            version,
            frame_rate,
            index,
            texture,
            shapes,
            transform,
            sprites,
            imports,
        })
    }
}

impl Decode for AnimationVersion {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        Ok(AnimationVersion(cursor.decode()?))
    }
}

impl Decode for Texture {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let name = cursor.decode::<String>()?;
        let crc = cursor.decode::<i32>()?;
        Ok(Texture { name, crc })
    }
}

impl Decode for Shape {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let id = cursor.decode::<i16>()?;
        let texture_index = cursor.decode::<i16>()?;
        let top = cursor.decode::<u16>()? as f32 / 65535f32;
        let left = cursor.decode::<u16>()? as f32 / 65535f32;
        let bottom = cursor.decode::<u16>()? as f32 / 65535f32;
        let right = cursor.decode::<u16>()? as f32 / 65535f32;
        let width = cursor.decode::<i16>()?;
        let height = cursor.decode::<i16>()?;
        let offset_x = cursor.decode::<f32>()?;
        let offset_y = cursor.decode::<f32>()?;
        Ok(Shape {
            id,
            texture_index,
            top,
            left,
            bottom,
            right,
            width,
            height,
            offset_x,
            offset_y,
        })
    }
}

impl Decode for TransformTable {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let colors = cursor.decode_prefixed::<u32, f32>()?;
        let rotations = cursor.decode_prefixed::<u32, f32>()?;
        let translations = cursor.decode_prefixed::<u32, f32>()?;
        let actions = cursor.decode_prefixed::<u32, Action>()?;
        Ok(TransformTable {
            colors,
            rotations,
            translations,
            actions,
        })
    }
}

impl Decode for Sprite {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let tag = cursor.decode::<i8>()?;
        let id = cursor.decode::<i16>()?;
        let flags = cursor.decode::<SpriteFlags>()?;
        let name = cursor.decode_opt::<String>(flags.has_name())?;
        let name_crc = cursor.decode::<i32>()?;
        let base_name_crc = cursor.decode::<i32>()?;
        let sprite_name = SpriteName {
            name,
            name_crc,
            base_name_crc,
        };
        let payload = match tag {
            1 => {
                let sprite_id = cursor.decode::<i16>()?;
                let action_info = cursor.decode_prefixed::<u16, i16>()?;
                Ok(SpritePayload::Single(sprite_id, action_info))
            }
            2 => Ok(SpritePayload::SingleNoAction(cursor.decode()?)),
            3 => {
                let sprite_ids = cursor.decode_prefixed::<u16, i16>()?;
                let action_info = cursor.decode_prefixed::<u16, i16>()?;
                Ok(SpritePayload::SingleFrame(sprite_ids, action_info))
            }
            4 => {
                let frame_pos = cursor.decode_prefixed::<u16, i32>()?;
                let sprite_ids = cursor.decode_prefixed::<u16, i16>()?;
                let action_info = cursor.decode_prefixed::<u16, i16>()?;
                Ok(SpritePayload::Indexed(frame_pos, sprite_ids, action_info))
            }
            other => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unexpected case: {}", other),
            )),
        };
        let frame_data = cursor.decode::<FrameData>()?;
        Ok(Sprite {
            id,
            name: sprite_name,
            flags,
            frame_data,
            payload: payload?,
        })
    }
}

impl Decode for SpriteFlags {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        Ok(SpriteFlags(cursor.decode()?))
    }
}

impl Decode for FrameData {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let tag = cursor.decode::<u8>()?;
        let size = cursor.decode::<u32>()? as usize;
        match tag {
            1 => {
                let mut buf = Vec::with_capacity(size);
                unsafe { buf.set_len(size) }
                cursor.read_exact(&mut buf)?;
                Ok(FrameData::Bytes(buf))
            }
            2 => Ok(FrameData::Shorts(cursor.decode_n(size)?)),
            4 => Ok(FrameData::Ints(cursor.decode_n(size)?)),
            other => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unexpected case: {}", other),
            )),
        }
    }
}

impl Decode for AnimationIndex {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let flags = cursor.decode::<AnimationFlags>()?;
        let scale = cursor.decode_opt(flags.has_scale())?;
        let render_radius = cursor.decode_opt(flags.has_render_radius())?;
        let file_names = if flags.has_extension() {
            Some(cursor.decode_prefixed::<u16, String>()?)
        } else {
            None
        };
        let parts_hidden_by = if flags.has_hiding_part() {
            Some(cursor.decode_prefixed::<u8, HideablePart>()?)
        } else {
            None
        };
        let parts_to_be_hidden = if flags.can_hide_part() {
            Some(cursor.decode_prefixed::<u8, HiddenPart>()?)
        } else {
            None
        };
        let extension = cursor.decode_opt::<AnimationExtension>(flags.is_extended())?;
        let animation_files = cursor.decode_prefixed::<u16, AnimationFile>()?;
        Ok(AnimationIndex {
            flags,
            scale,
            render_radius,
            file_names,
            animation_files,
            parts_to_be_hidden,
            parts_hidden_by,
            extension,
        })
    }
}

impl Decode for AnimationFlags {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        Ok(AnimationFlags(cursor.decode()?))
    }
}

impl Decode for HideablePart {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let crc_key = cursor.decode::<i32>()?;
        let crc_to_hide = cursor.decode::<i32>()?;
        Ok(HideablePart { crc_key, crc_to_hide })
    }
}

impl Decode for HiddenPart {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let item_name = cursor.decode::<String>()?;
        let crc_key = cursor.decode::<i32>()?;
        Ok(HiddenPart { item_name, crc_key })
    }
}

impl Decode for AnimationExtension {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let flags = cursor.decode::<i32>()?;
        let heights = if flags & 0x1 == 0x1 {
            let count = cursor.decode::<u16>()?;
            let mut map = HashMap::with_capacity(count.into());
            for _ in 0..count {
                let key = cursor.decode::<i32>()?;
                let height = cursor.decode::<i8>()? + 1;
                map.insert(key, height);
            }
            Some(map)
        } else {
            None
        };
        let highlight_color = cursor.decode_opt(flags & 0x2 == 0x2)?;
        Ok(AnimationExtension {
            heights,
            highlight_color,
        })
    }
}

impl Decode for Color {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let red = cursor.decode::<f32>()?;
        let green = cursor.decode::<f32>()?;
        let blue = cursor.decode::<f32>()?;
        Ok(Color {
            red,
            green,
            blue,
            alpha: 1f32,
        })
    }
}

impl Decode for AnimationFile {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let name = cursor.decode::<String>()?;
        let crc = cursor.decode::<i32>()?;
        let file_index = cursor.decode::<i16>()?;
        Ok(AnimationFile { name, crc, file_index })
    }
}

impl Decode for Action {
    fn decode<R: io::Read + Sized>(cursor: &mut R) -> io::Result<Self> {
        let id = cursor.decode::<u8>()?;
        let param_count = cursor.decode::<u8>()?;
        match id {
            1 => {
                let name = cursor.decode::<String>()?;
                let percent = cursor.decode_opt::<u8>(param_count == 2)?;
                Ok(Action::GoTo(name, percent))
            }
            2 => Ok(Action::GoToStatic),
            3 => Ok(Action::RunScript(cursor.decode::<String>()?)),
            4 => {
                let first = cursor.decode::<String>()?;
                if &first == "#optimized" {
                    let count = (param_count - 1) / 2;
                    let mut names = vec![first];
                    for _ in 0..count {
                        names.push(cursor.decode()?)
                    }
                    let percents = cursor.decode_n::<u8>(count.into())?;
                    Ok(Action::GoToRandom(names, percents))
                } else {
                    let names = cursor.decode_n::<String>((param_count - 1).into())?;
                    Ok(Action::GoToRandom(names, vec![]))
                }
            }
            5 => Ok(Action::Hit),
            6 => Ok(Action::Delete),
            7 => Ok(Action::End),
            8 => {
                let count = (param_count - 1) / 2;
                let mut previous = Vec::with_capacity(count.into());
                let mut next = Vec::with_capacity(count.into());
                for _ in 0..count {
                    previous.push(cursor.decode()?);
                    next.push(cursor.decode()?);
                }
                let default = cursor.decode_opt::<String>(param_count % 2 == 1)?;
                Ok(Action::GoToIfPrevious(previous, next, default))
            }
            9 => {
                let particle_id = cursor.decode::<i32>()?;
                let offset_x = cursor.decode_opt::<i16>(param_count > 1)?;
                let offset_y = cursor.decode_opt::<i16>(param_count > 2)?;
                let offset_z = cursor.decode_opt::<i16>(param_count > 3)?;
                Ok(Action::AddParticle(particle_id, offset_x, offset_y, offset_z))
            }
            10 => Ok(Action::SetRadius(cursor.decode()?)),
            other => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unexpected case: {}", other),
            )),
        }
    }
}

impl Decode for Import {
    fn decode<R: io::Read>(cursor: &mut R) -> io::Result<Self> {
        let id = cursor.decode::<i16>()?;
        let name = cursor.decode::<String>()?;
        let crc = cursor.decode::<i32>()?;
        Ok(Import { id, name, crc })
    }
}
