use crate::frame_reader::FrameReader;
use crate::types::{Animation, Color, Operation, Shape, Sprite, SpritePayload, TransformTable};
use euclid::{vec2, Transform2D};

pub trait Renderer {
    fn render(&mut self, shape: &Shape, transform: Transformation) -> ();

    fn draw_sprite(&mut self, animation: &Animation, sprite: &Sprite, transform: Transformation, frame: u32) {
        let empty_table = &TransformTable::EMPTY;
        let table = animation.transform.as_ref().unwrap_or(empty_table);
        match &sprite.payload {
            SpritePayload::Single(sprite_id, _) => {
                let ops = FrameReader::new(&sprite.frame_data, table).read_operations().unwrap();
                let t = ops.iter().fold(transform, |acc, op| acc.apply(op));
                self.draw_sprite_by_id(animation, *sprite_id, t, frame);
            }
            SpritePayload::SingleNoAction(sprite_id) => {
                let ops = FrameReader::new(&sprite.frame_data, table).read_operations().unwrap();
                let t = ops.iter().fold(transform, |acc, op| acc.apply(op));
                self.draw_sprite_by_id(animation, *sprite_id, t, frame)
            }
            SpritePayload::SingleFrame(sprite_ids, _) => {
                let mut reader = FrameReader::new(&sprite.frame_data, table);
                for sprite_id in sprite_ids {
                    let ops = reader.read_operations().unwrap();
                    let t = ops.iter().fold(transform.clone(), |acc, op| acc.apply(op));
                    self.draw_sprite_by_id(animation, *sprite_id, t, frame)
                }
            }
            SpritePayload::Indexed(frame_pos, sprite_info, action_info) => {
                let mult = if action_info.len() == 0 { 2 } else { 3 };
                let index = (frame as usize % sprite.frame_count()) * mult;
                let offset = *frame_pos.get(index).unwrap() as usize;
                let current = *frame_pos.get(index + 1).unwrap() as usize;
                let count = *sprite_info.get(current).unwrap() as usize;
                let mut reader = FrameReader::new(&sprite.frame_data, table);
                reader.seek(offset);
                for sprite_id in sprite_info.iter().skip(current + 1).take(count) {
                    let ops = reader.read_operations().unwrap();
                    let t = ops.iter().fold(transform.clone(), |acc, op| acc.apply(op));
                    self.draw_sprite_by_id(animation, *sprite_id, t, frame)
                }
            }
        }
    }

    fn draw_sprite_by_id(&mut self, animation: &Animation, id: i16, transform: Transformation, frame: u32) {
        match animation.sprites.get(&id) {
            None => match animation.shapes.get(&id) {
                None => (),
                Some(shape) => {
                    let t = Transformation {
                        position: transform
                            .position
                            .pre_translate(vec2(shape.offset_x, shape.offset_y))
                            .post_scale(0.005, -0.005)
                            .pre_scale(shape.width as f32, shape.height as f32),
                        color: transform.color,
                    };
                    self.render(shape, t);
                }
            },
            Some(sprite) => self.draw_sprite(animation, sprite, transform, frame),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transformation {
    pub position: Transform2D<f32, (), ()>,
    pub color: Color,
}

impl Transformation {
    pub fn identity() -> Transformation {
        Transformation {
            position: Transform2D::identity(),
            color: Color {
                red: 1f32,
                green: 1f32,
                blue: 1f32,
                alpha: 1f32,
            },
        }
    }

    pub fn apply(self, operation: &Operation) -> Transformation {
        match *operation {
            Operation::Translate(x, y) => Transformation {
                position: self.position.post_translate(vec2(x, y)),
                color: self.color,
            },
            Operation::Rotate(rx0, ry0, rx1, ry1) => {
                let matrix = Transform2D::column_major(rx0, ry0, 0f32, rx1, ry1, 0f32);
                Transformation {
                    position: self.position.post_transform(&matrix),
                    color: self.color,
                }
            }
            Operation::Scale(sx, sy) => Transformation {
                position: self.position.post_scale(sx, sy),
                color: self.color,
            },
            Operation::ColorMultiply(r, g, b, a) => Transformation {
                position: self.position,
                color: Color {
                    red: self.color.red * r,
                    green: self.color.green * g,
                    blue: self.color.blue * b,
                    alpha: self.color.alpha * a,
                },
            },
            Operation::ColorAdd(r, g, b, a) => Transformation {
                position: self.position,
                color: Color {
                    red: self.color.red + r,
                    green: self.color.green + g,
                    blue: self.color.blue + b,
                    alpha: self.color.alpha + a,
                },
            },
        }
    }
}
