use crate::render::SpriteTransform;
use crate::types::{FrameData, TransformTable};

pub struct FrameReader<'a> {
    data: &'a FrameData,
    transform: &'a TransformTable,
    position: usize,
}

impl<'a> FrameReader<'a> {
    pub fn new(data: &'a FrameData, transform: &'a TransformTable) -> FrameReader<'a> {
        FrameReader {
            data,
            transform,
            position: 0,
        }
    }

    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }

    pub fn read_transformation(&mut self) -> Option<SpriteTransform> {
        let tag = self.read_int()?;
        match tag {
            0 => Some(SpriteTransform::identity()),
            1 => self.read_rotation(),
            2 => self.read_translation(),
            3 => Some(self.read_rotation()?.combine(self.read_translation()?)),
            4 => self.read_color_multiply(),
            5 => Some(self.read_color_multiply()?.combine(self.read_rotation()?)),
            6 => Some(self.read_color_multiply()?.combine(self.read_translation()?)),
            7 => Some(
                self.read_color_multiply()?
                    .combine(self.read_rotation()?)
                    .combine(self.read_translation()?),
            ),
            8 => self.read_color_add(),
            9 => Some(self.read_color_add()?.combine(self.read_rotation()?)),
            10 => Some(self.read_color_add()?.combine(self.read_translation()?)),
            11 => Some(
                self.read_color_add()?
                    .combine(self.read_rotation()?)
                    .combine(self.read_translation()?),
            ),
            12 => Some(self.read_color_multiply()?.combine(self.read_color_add()?)),
            13 => Some(
                self.read_color_multiply()?
                    .combine(self.read_color_add()?)
                    .combine(self.read_rotation()?),
            ),
            14 => Some(
                self.read_color_multiply()?
                    .combine(self.read_color_add()?)
                    .combine(self.read_translation()?),
            ),
            15 => Some(
                self.read_color_multiply()?
                    .combine(self.read_color_add()?)
                    .combine(self.read_rotation()?)
                    .combine(self.read_translation()?),
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

    #[inline]
    fn read_translation(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let x = self.transform.translations.get(offset)?.clone();
        let y = self.transform.translations.get(offset + 1)?.clone();
        Some(SpriteTransform::translate(x, y))
    }

    #[inline]
    fn read_rotation(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let rx0 = self.transform.rotations.get(offset)?.clone();
        let rx1 = self.transform.rotations.get(offset + 1)?.clone();
        let ry0 = self.transform.rotations.get(offset + 2)?.clone();
        let ry1 = self.transform.rotations.get(offset + 3)?.clone();
        Some(SpriteTransform::rotate(rx0, rx1, ry0, ry1))
    }

    #[inline]
    fn read_color_multiply(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let r = self.transform.colors.get(offset)?.clone();
        let g = self.transform.colors.get(offset + 1)?.clone();
        let b = self.transform.colors.get(offset + 2)?.clone();
        let a = self.transform.colors.get(offset + 3)?.clone();
        Some(SpriteTransform::color_multiply(r, g, b, a))
    }

    #[inline]
    fn read_color_add(&mut self) -> Option<SpriteTransform> {
        let offset = self.read_int()? as usize;
        let r = self.transform.colors.get(offset)?.clone();
        let g = self.transform.colors.get(offset + 1)?.clone();
        let b = self.transform.colors.get(offset + 2)?.clone();
        let a = self.transform.colors.get(offset + 3)?.clone();
        Some(SpriteTransform::color_add(r, g, b, a))
    }
}
