use crate::frame_reader::FrameReader;
use crate::types::{Animation, Color, Operation, Shape, Sprite, SpritePayload, TransformTable, Transformation};
use euclid::Transform2D;

pub trait Renderer {
    fn render(&mut self, shape: &Shape, transformation: Transformation) -> ();

    fn draw_sprite(&mut self, animation: &Animation, sprite: &Sprite, operations: Vec<Operation>, frame: u32) {
        let empty_table = &TransformTable::EMPTY;
        let table = animation.transform.as_ref().unwrap_or(empty_table);
        match &sprite.payload {
            SpritePayload::Single(sprite_id, _) => {
                let mut ops = FrameReader::new(&sprite.frame_data, table).read_operations().unwrap();
                ops.extend(operations);
                self.draw_sprite_by_id(animation, *sprite_id, ops, frame);
            }
            SpritePayload::SingleNoAction(sprite_id) => {
                let mut ops = FrameReader::new(&sprite.frame_data, table).read_operations().unwrap();
                ops.extend(operations);
                self.draw_sprite_by_id(animation, *sprite_id, ops, frame)
            }
            SpritePayload::SingleFrame(sprite_ids, _) => {
                let mut reader = FrameReader::new(&sprite.frame_data, table);
                for sprite_id in sprite_ids {
                    let mut ops = reader.read_operations().unwrap();
                    ops.extend(operations.clone());
                    self.draw_sprite_by_id(animation, *sprite_id, ops, frame)
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
                    let mut ops = reader.read_operations().unwrap();
                    ops.extend(operations.clone());
                    self.draw_sprite_by_id(animation, *sprite_id, ops, frame)
                }
            }
        }
    }

    fn draw_sprite_by_id(&mut self, animation: &Animation, id: i16, operations: Vec<Operation>, frame: u32) {
        match animation.sprites.get(&id) {
            None => match animation.shapes.get(&id) {
                None => (),
                Some(shape) => {
                    let mut ops = operations.clone();
                    ops.push(Operation::Translate(0., 25.));
                    ops.push(Operation::Scale(0.005, -0.005));
                    let transform = compute_transformation(shape, ops);
                    self.render(shape, transform);
                }
            },
            Some(sprite) => self.draw_sprite(animation, sprite, operations, frame),
        }
    }
}

fn compute_transformation(shape: &Shape, operations: Vec<Operation>) -> Transformation {
    let initial = Transformation {
        position: Transform2D::create_translation(
            shape.offset_x / shape.width as f32,
            shape.offset_y / shape.height as f32,
        ),
        color: Color {
            red: 1f32,
            green: 1f32,
            blue: 1f32,
            alpha: 1f32,
        },
    };

    operations
        .iter()
        .fold(initial, |acc, op| apply_operation(acc, shape, op))
}

fn apply_operation(transformation: Transformation, shape: &Shape, operation: &Operation) -> Transformation {
    match operation {
        Operation::Translate(x, y) => Transformation {
            position: transformation
                .position
                .post_translate(euclid::vec2(*x / shape.width as f32, *y / shape.height as f32)),
            color: transformation.color,
        },
        Operation::Rotate(rx0, ry0, rx1, ry1) => {
            let matrix = Transform2D::column_major(*rx0, *ry0, 0f32, *rx1, *ry1, 0f32);
            Transformation {
                position: transformation.position.post_transform(&matrix),
                color: transformation.color,
            }
        }
        Operation::Scale(sx, sy) => Transformation {
            position: transformation
                .position
                .post_scale(sx * shape.width as f32, sy * shape.height as f32),
            color: transformation.color,
        },
        Operation::ColorMultiply(r, g, b, a) => {
            let color = Color {
                red: transformation.color.red * *r,
                green: transformation.color.green * *g,
                blue: transformation.color.blue * *b,
                alpha: transformation.color.alpha * *a,
            };
            Transformation {
                position: transformation.position,
                color,
            }
        }
        Operation::ColorAdd(r, g, b, a) => {
            let color = Color {
                red: transformation.color.red + *r,
                green: transformation.color.green + *g,
                blue: transformation.color.blue + *b,
                alpha: transformation.color.alpha + *a,
            };
            Transformation {
                position: transformation.position,
                color,
            }
        }
    }
}
