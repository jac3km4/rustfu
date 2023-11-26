use notan::app::BlendMode;
use notan::draw::{Draw, DrawImages, DrawTransform};
use notan::graphics::Texture;
use notan::math::Mat3;

use crate::render::{Render, SpriteTransform};
use crate::types::Shape;

#[derive(Debug)]
pub struct NotanBackend {
    draw: Draw,
    atlas: Texture,
}

impl NotanBackend {
    #[inline]
    pub fn new(draw: Draw, atlas: Texture) -> Self {
        Self { draw, atlas }
    }

    #[inline]
    pub fn swap(&mut self, draw: Draw) -> Draw {
        std::mem::replace(&mut self.draw, draw)
    }

    #[inline]
    pub fn texture(&self) -> &Texture {
        &self.atlas
    }

    #[inline]
    pub fn draw_mut(&mut self) -> &mut Draw {
        &mut self.draw
    }

    #[inline]
    pub fn clone_with_draw(&self, draw: Draw) -> Self {
        Self {
            draw,
            atlas: self.atlas.clone(),
        }
    }
}

impl Render for NotanBackend {
    fn render(&mut self, shape: &Shape, transform: SpriteTransform) {
        let [x0, y0, x1, y1, x2, y2] = transform.position.to_array();
        let mat = Mat3::from_cols_array(&[x0, y0, 0., x1, y1, 0., x2, y2, 0.]);
        let color = transform.color.into_color();

        self.draw
            .image(&self.atlas)
            .position(shape.offset_x, shape.offset_y)
            .size(shape.width as _, shape.height as _)
            .crop(
                (
                    shape.left * self.atlas.width(),
                    shape.top * self.atlas.height(),
                ),
                (
                    (shape.right - shape.left) * self.atlas.width(),
                    (shape.bottom - shape.top) * self.atlas.height(),
                ),
            )
            .flip_y(true)
            .transform(mat)
            .blend_mode(BlendMode::OVER)
            .color(<[f32; 4]>::from(color).into());
    }
}
