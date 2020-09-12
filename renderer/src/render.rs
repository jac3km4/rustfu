use euclid::Transform2D;

use crate::frame_reader::FrameReader;
use crate::types::{Animation, Color, Shape, Sprite, SpritePayload, TransformTable};

pub trait Render {
    fn render(&mut self, shape: &Shape, transform: SpriteTransform) -> ();

    fn render_sprite(&mut self, animation: &Animation, sprite: &Sprite, transform: SpriteTransform, frame: u32) {
        let empty_table = &TransformTable::EMPTY;
        let table = animation.transform.as_ref().unwrap_or(empty_table);
        let mut reader = FrameReader::new(&sprite.frame_data, table);
        match &sprite.payload {
            SpritePayload::Single(sprite_id, _) => {
                self.render_by_id(animation, *sprite_id, &transform, &mut reader, frame);
            }
            SpritePayload::SingleNoAction(sprite_id) => {
                self.render_by_id(animation, *sprite_id, &transform, &mut reader, frame);
            }
            SpritePayload::SingleFrame(sprite_ids, _) => {
                for sprite_id in sprite_ids {
                    self.render_by_id(animation, *sprite_id, &transform, &mut reader, frame);
                }
            }
            SpritePayload::Indexed(frame_pos, sprite_info, action_info) => {
                let mult = if action_info.len() == 0 { 2 } else { 3 };
                let index = (frame as usize % sprite.frame_count()) * mult;
                let offset = *frame_pos.get(index).unwrap() as usize;
                let current = *frame_pos.get(index + 1).unwrap() as usize;
                let count = *sprite_info.get(current).unwrap() as usize;
                reader.seek(offset);
                for sprite_id in sprite_info.iter().skip(current + 1).take(count) {
                    self.render_by_id(animation, *sprite_id, &transform, &mut reader, frame);
                }
            }
        }
    }

    fn render_by_id(&mut self, anm: &Animation, id: i16, parent: &SpriteTransform, reader: &mut FrameReader, i: u32) {
        let transform = reader.read_transformation().unwrap().combine(parent.clone());
        match anm.sprites.get(&id) {
            Some(sprite) => self.render_sprite(anm, sprite, transform, i),
            None => match anm.shapes.get(&id) {
                Some(shape) => self.render(shape, transform),
                None => (),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpriteTransform {
    pub position: Transform2D<f32, (), ()>,
    pub color: ColorTransform,
}

impl SpriteTransform {
    #[inline]
    pub fn identity() -> SpriteTransform {
        SpriteTransform {
            position: Transform2D::identity(),
            color: ColorTransform::identity(),
        }
    }

    #[inline]
    pub fn combine(self, other: SpriteTransform) -> SpriteTransform {
        SpriteTransform {
            position: self.position.then(&other.position),
            color: self.color.combine(other.color),
        }
    }

    #[inline]
    pub fn translate(x: f32, y: f32) -> SpriteTransform {
        SpriteTransform {
            position: Transform2D::translation(x, y),
            color: ColorTransform::identity(),
        }
    }

    #[inline]
    pub fn rotate(rx0: f32, ry0: f32, rx1: f32, ry1: f32) -> SpriteTransform {
        SpriteTransform {
            position: Transform2D::new(rx0, ry0, rx1, ry1, 0f32, 0f32),
            color: ColorTransform::identity(),
        }
    }

    #[inline]
    pub fn scale(sx: f32, sy: f32) -> SpriteTransform {
        SpriteTransform {
            position: Transform2D::scale(sx, sy),
            color: ColorTransform::identity(),
        }
    }

    #[inline]
    pub fn color_multiply(red: f32, green: f32, blue: f32, alpha: f32) -> SpriteTransform {
        SpriteTransform {
            position: Transform2D::identity(),
            color: ColorTransform::Multiply(red, green, blue, alpha),
        }
    }

    #[inline]
    pub fn color_add(red: f32, green: f32, blue: f32, alpha: f32) -> SpriteTransform {
        SpriteTransform {
            position: Transform2D::identity(),
            color: ColorTransform::Add(red, green, blue, alpha),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColorTransform {
    Multiply(f32, f32, f32, f32),
    Add(f32, f32, f32, f32),
    Combine(Box<ColorTransform>, Box<ColorTransform>),
}

impl ColorTransform {
    #[inline]
    pub fn identity() -> ColorTransform {
        ColorTransform::Add(0., 0., 0., 0.)
    }

    #[inline]
    pub fn combine(self, other: ColorTransform) -> ColorTransform {
        match (self, other) {
            (ColorTransform::Multiply(lr, lg, lb, la), ColorTransform::Multiply(rr, rg, rb, ra)) => {
                ColorTransform::Multiply(lr * rr, lg * rg, lb * rb, la * ra)
            }
            (ColorTransform::Add(lr, lg, lb, la), ColorTransform::Add(rr, rg, rb, ra)) => {
                ColorTransform::Add(lr + rr, lg + rg, lb + rb, la + ra)
            }
            (l, r) => ColorTransform::Combine(Box::new(l), Box::new(r)),
        }
    }

    pub fn fold(self, color: Color) -> Color {
        match self {
            ColorTransform::Multiply(r, g, b, a) => Color {
                red: color.red * r,
                green: color.green * g,
                blue: color.blue * b,
                alpha: color.alpha * a,
            },
            ColorTransform::Add(r, g, b, a) => Color {
                red: color.red + r,
                green: color.green + g,
                blue: color.blue + b,
                alpha: color.alpha + a,
            },
            ColorTransform::Combine(l, r) => r.fold(l.fold(color)),
        }
    }

    pub fn color(self) -> Color {
        let initial = Color {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        };
        self.fold(initial)
    }
}
