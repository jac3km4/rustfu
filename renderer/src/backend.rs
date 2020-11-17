use crate::render::{Render, SpriteTransform};
use crate::types::{Animation, Shape, Sprite};
use quicksilver::geom::{Rectangle, Transform, Vector};
use quicksilver::graphics::blend::{BlendChannel, BlendFactor, BlendFunction, BlendInput, BlendMode};
use quicksilver::graphics::{Image, PixelFormat};
use quicksilver::{Graphics, Window};
use std::collections::HashMap;
use std::rc::Rc;

pub struct QuicksilverBackend {
    graphics: Graphics,
    sprites: HashMap<i16, AtlasSprite>,
    atlas: Option<Rc<Atlas>>,
}

impl QuicksilverBackend {
    pub fn new(mut graphics: Graphics, window: &Window) -> QuicksilverBackend {
        let blend_mode = BlendMode {
            function: BlendFunction::Same {
                source: BlendFactor::One,
                destination: BlendFactor::Color {
                    input: BlendInput::Source,
                    channel: BlendChannel::Alpha,
                    is_inverse: true,
                },
            },
            ..BlendMode::default()
        };
        graphics.set_blend_mode(Some(blend_mode));
        graphics.set_view(Self::calculate_viewport(window.size()));

        QuicksilverBackend {
            graphics,
            sprites: HashMap::new(),
            atlas: None,
        }
    }

    pub fn set_atlas(&mut self, texture: image::RgbaImage) {
        let width = texture.width();
        let height = texture.height();
        let data = texture.into_raw();
        let image = Image::from_raw(&self.graphics, Some(&data), width, height, PixelFormat::RGBA).unwrap();
        let atlas = Atlas::new(image, Vector::new(width as f32, height as f32));
        self.sprites.clear();
        self.atlas = Some(Rc::new(atlas));
    }

    pub fn set_viewport(&mut self, dimensions: Vector) {
        self.graphics.set_view(Self::calculate_viewport(dimensions))
    }

    pub fn render(&mut self, animation: &Animation, sprite: &Sprite, frame: u32) {
        let scale = animation.index.as_ref().and_then(|i| i.scale).unwrap_or(1.);
        self.render_sprite(animation, sprite, SpriteTransform::scale(scale, scale), frame)
    }

    pub fn context(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    fn calculate_viewport(dimensions: Vector) -> Transform {
        let offset = Transform::translate(Vector::new(dimensions.x / 2., dimensions.y / 4. * 3.));
        let scale = Transform::scale(dimensions / dimensions.x.min(dimensions.y) * 2.);
        offset * scale
    }
}

impl Render for QuicksilverBackend {
    fn render(&mut self, shape: &Shape, transform: SpriteTransform) -> () {
        let image = self.atlas.as_ref().expect("No atlas set").clone();
        let sprite = self
            .sprites
            .entry(shape.id)
            .or_insert_with(|| AtlasSprite::new(image, shape));

        sprite.render(&mut self.graphics, transform)
    }
}

struct Atlas {
    image: Image,
    size: Vector,
}

impl Atlas {
    fn new(image: Image, size: Vector) -> Atlas {
        Atlas { image, size }
    }
}

struct AtlasSprite {
    atlas: Rc<Atlas>,
    region: Rectangle,
    location: Rectangle,
}

impl AtlasSprite {
    fn new(atlas: Rc<Atlas>, shape: &Shape) -> AtlasSprite {
        let width = shape.width as f32;
        let height = shape.height as f32;
        let offset = Vector::new(shape.offset_x, shape.offset_y + height);
        let size = Vector::new(width, -height);
        let location = Rectangle::new(offset, size);

        let atlas_offset = Vector::new(shape.left * atlas.size.x, shape.top * atlas.size.y);
        let atlas_size = Vector::new(
            (shape.right - shape.left) * atlas.size.x,
            (shape.bottom - shape.top) * atlas.size.y,
        );
        let region = Rectangle::new(atlas_offset, atlas_size);

        AtlasSprite {
            atlas,
            region,
            location,
        }
    }

    fn render(&self, graphics: &mut Graphics, transformation: SpriteTransform) {
        graphics.set_transform(transformation.position);
        graphics.draw_subimage_tinted(
            &self.atlas.image,
            self.region,
            self.location,
            transformation.color.color(),
        );
    }
}
