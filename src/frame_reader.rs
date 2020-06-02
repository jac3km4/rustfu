use crate::types::{FrameData, Operation, TransformTable};

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

    pub fn read_operations(&mut self) -> Option<Vec<Operation>> {
        let tag = self.read_int()?;
        match tag {
            0 => Some(vec![]),
            1 => Some(vec![self.read_rotation()?]),
            2 => Some(vec![self.read_translation()?]),
            3 => Some(vec![self.read_rotation()?, self.read_translation()?]),
            4 => Some(vec![self.read_color_multiply()?]),
            5 => Some(vec![self.read_color_multiply()?, self.read_rotation()?]),
            6 => Some(vec![self.read_color_multiply()?, self.read_translation()?]),
            7 => Some(vec![
                self.read_color_multiply()?,
                self.read_rotation()?,
                self.read_translation()?,
            ]),
            8 => Some(vec![self.read_color_add()?]),
            9 => Some(vec![self.read_color_add()?, self.read_rotation()?]),
            10 => Some(vec![self.read_color_add()?, self.read_translation()?]),
            11 => Some(vec![
                self.read_color_add()?,
                self.read_rotation()?,
                self.read_translation()?,
            ]),
            12 => Some(vec![self.read_color_multiply()?, self.read_color_add()?]),
            13 => Some(vec![
                self.read_color_multiply()?,
                self.read_color_add()?,
                self.read_rotation()?,
            ]),
            14 => Some(vec![
                self.read_color_multiply()?,
                self.read_color_add()?,
                self.read_translation()?,
            ]),
            15 => Some(vec![
                self.read_color_multiply()?,
                self.read_color_add()?,
                self.read_rotation()?,
                self.read_translation()?,
            ]),
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

    fn read_translation(&mut self) -> Option<Operation> {
        let offset = self.read_int()? as usize;
        let x = self.transform.translations.get(offset)?.clone();
        let y = self.transform.translations.get(offset + 1)?.clone();
        Some(Operation::Translate(x, y))
    }

    fn read_rotation(&mut self) -> Option<Operation> {
        let offset = self.read_int()? as usize;
        let rx0 = self.transform.rotations.get(offset)?.clone();
        let rx1 = self.transform.rotations.get(offset + 1)?.clone();
        let ry0 = self.transform.rotations.get(offset + 2)?.clone();
        let ry1 = self.transform.rotations.get(offset + 3)?.clone();
        Some(Operation::Rotate(rx0, rx1, ry0, ry1))
    }

    fn read_color_multiply(&mut self) -> Option<Operation> {
        let offset = self.read_int()? as usize;
        let r = self.transform.colors.get(offset)?.clone();
        let g = self.transform.colors.get(offset + 1)?.clone();
        let b = self.transform.colors.get(offset + 2)?.clone();
        let a = self.transform.colors.get(offset + 3)?.clone();
        Some(Operation::ColorMultiply(r, g, b, a))
    }

    fn read_color_add(&mut self) -> Option<Operation> {
        let offset = self.read_int()? as usize;
        let r = self.transform.colors.get(offset)?.clone();
        let g = self.transform.colors.get(offset + 1)?.clone();
        let b = self.transform.colors.get(offset + 2)?.clone();
        let a = self.transform.colors.get(offset + 3)?.clone();
        Some(Operation::ColorAdd(r, g, b, a))
    }
}
