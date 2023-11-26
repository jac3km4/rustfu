use crate::render::SpriteTransform;
use crate::types::{FrameData, TransformTable};

pub struct FrameReader<'a> {
    data: &'a FrameData,
    transform: &'a TransformTable,
    position: usize,
}

impl<'a> FrameReader<'a> {
    #[inline]
    pub fn new(data: &'a FrameData, transform: &'a TransformTable) -> FrameReader<'a> {
        FrameReader {
            data,
            transform,
            position: 0,
        }
    }

    #[inline]
    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }

    pub fn read_transformation(&mut self) -> Option<SpriteTransform> {
        let tag = self.read_int()?;
        match tag {
            0 => Some(SpriteTransform::identity()),
            1 => self.read_rotation(),
            2 => self.read_translation(),
            3 => Some(self.read_rotation()?.combine(&self.read_translation()?)),
            4 => self.read_color_multiply(),
            5 => Some(self.read_color_multiply()?.combine(&self.read_rotation()?)),
            6 => Some(
                self.read_color_multiply()?
                    .combine(&self.read_translation()?),
            ),
            7 => Some(
                self.read_color_multiply()?
                    .combine(&self.read_rotation()?)
                    .combine(&self.read_translation()?),
            ),
            8 => self.read_color_add(),
            9 => Some(self.read_color_add()?.combine(&self.read_rotation()?)),
            10 => Some(self.read_color_add()?.combine(&self.read_translation()?)),
            11 => Some(
                self.read_color_add()?
                    .combine(&self.read_rotation()?)
                    .combine(&self.read_translation()?),
            ),
            12 => Some(self.read_color_multiply()?.combine(&self.read_color_add()?)),
            13 => Some(
                self.read_color_multiply()?
                    .combine(&self.read_color_add()?)
                    .combine(&self.read_rotation()?),
            ),
            14 => Some(
                self.read_color_multiply()?
                    .combine(&self.read_color_add()?)
                    .combine(&self.read_translation()?),
            ),
            15 => Some(
                self.read_color_multiply()?
                    .combine(&self.read_color_add()?)
                    .combine(&self.read_rotation()?)
                    .combine(&self.read_translation()?),
            ),
            _ => None,
        }
    }

    fn read_int(&mut self) -> Option<u32> {
        let res = match &self.data {
            FrameData::Ints(vec) => *vec.get(self.position)?,
            FrameData::Shorts(vec) => (*vec.get(self.position)?).into(),
            FrameData::Bytes(vec) => (*vec.get(self.position)?).into(),
        };
        self.position += 1;
        Some(res)
    }

    fn read_translation(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let x = *self.transform.translations.get(offset)?;
        let y = *self.transform.translations.get(offset + 1)?;
        Some(SpriteTransform::translate(x, y))
    }

    fn read_rotation(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let rx0 = *self.transform.rotations.get(offset)?;
        let rx1 = *self.transform.rotations.get(offset + 1)?;
        let ry0 = *self.transform.rotations.get(offset + 2)?;
        let ry1 = *self.transform.rotations.get(offset + 3)?;
        Some(SpriteTransform::rotate(rx0, rx1, ry0, ry1))
    }

    fn read_color_multiply(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let r = *self.transform.colors.get(offset)?;
        let g = *self.transform.colors.get(offset + 1)?;
        let b = *self.transform.colors.get(offset + 2)?;
        let a = *self.transform.colors.get(offset + 3)?;
        Some(SpriteTransform::color_multiply(r, g, b, a))
    }

    fn read_color_add(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let r = *self.transform.colors.get(offset)?;
        let g = *self.transform.colors.get(offset + 1)?;
        let b = *self.transform.colors.get(offset + 2)?;
        let a = *self.transform.colors.get(offset + 3)?;
        Some(SpriteTransform::color_add(r, g, b, a))
    }
}
