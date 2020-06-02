use crate::frame_reader::FrameReader;
use crate::types::{Animation, Operation, Shape, Sprite, SpritePayload, TransformTable, Transformation};
use cgmath::{Matrix3, Vector4};

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
        position: Matrix3::new(
            1f32,
            0f32,
            0f32,
            0f32,
            1f32,
            0f32,
            shape.offset_x / shape.width as f32,
            shape.offset_y / shape.height as f32,
            1f32,
        ),
        color: Vector4 {
            x: 1f32,
            y: 1f32,
            z: 1f32,
            w: 1f32,
        },
    };

    operations
        .iter()
        .fold(initial, |acc, op| apply_operation(acc, shape, op))
}

fn apply_operation(transformation: Transformation, shape: &Shape, operation: &Operation) -> Transformation {
    match operation {
        Operation::Translate(x, y) => {
            let matrix = Matrix3::new(
                1f32,
                0f32,
                0f32,
                0f32,
                1f32,
                0f32,
                *x / shape.width as f32,
                *y / shape.height as f32,
                1f32,
            );
            Transformation {
                position: multiply(transformation.position, matrix),
                color: transformation.color,
            }
        }
        Operation::Rotate(rx0, ry0, rx1, ry1) => {
            let matrix = Matrix3::new(*rx0, *ry0, 0f32, *rx1, *ry1, 0f32, 0f32, 0f32, 1f32);
            Transformation {
                position: multiply(transformation.position, matrix),
                color: transformation.color,
            }
        }
        Operation::Scale(sx, sy) => {
            let matrix = Matrix3::new(
                sx * shape.width as f32,
                0f32,
                0f32,
                0f32,
                sy * shape.height as f32,
                0f32,
                0f32,
                0f32,
                1f32,
            );
            Transformation {
                position: multiply(transformation.position, matrix),
                color: transformation.color,
            }
        }
        Operation::ColorMultiply(r, g, b, a) => {
            let color = Vector4 {
                x: transformation.color.x * *r,
                y: transformation.color.y * *g,
                z: transformation.color.z * *b,
                w: transformation.color.w * *a,
            };
            Transformation {
                position: transformation.position,
                color,
            }
        }
        Operation::ColorAdd(r, g, b, a) => {
            let color = Vector4 {
                x: transformation.color.x + *r,
                y: transformation.color.y + *g,
                z: transformation.color.z + *b,
                w: transformation.color.w + *a,
            };
            Transformation {
                position: transformation.position,
                color,
            }
        }
    }
}

fn multiply(a: Matrix3<f32>, b: Matrix3<f32>) -> Matrix3<f32> {
    let a00 = a.x.x;
    let a01 = a.x.y;
    let a02 = a.x.z;
    let a10 = a.y.x;
    let a11 = a.y.y;
    let a12 = a.y.z;
    let a20 = a.z.x;
    let a21 = a.z.y;
    let a22 = a.z.z;
    let b00 = b.x.x;
    let b01 = b.x.y;
    let b02 = b.x.z;
    let b10 = b.y.x;
    let b11 = b.y.y;
    let b12 = b.y.z;
    let b20 = b.z.x;
    let b21 = b.z.y;
    let b22 = b.z.z;
    Matrix3::new(
        a00 * b00 + a01 * b10 + a02 * b20,
        a00 * b01 + a01 * b11 + a02 * b21,
        a00 * b02 + a01 * b12 + a02 * b22,
        a10 * b00 + a11 * b10 + a12 * b20,
        a10 * b01 + a11 * b11 + a12 * b21,
        a10 * b02 + a11 * b12 + a12 * b22,
        a20 * b00 + a21 * b10 + a22 * b20,
        a20 * b01 + a21 * b11 + a22 * b21,
        a20 * b02 + a21 * b12 + a22 * b22,
    )
}
