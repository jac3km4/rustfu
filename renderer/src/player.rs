use std::sync::Arc;

use crate::render::{Render, SpriteTransform};
use crate::types::{Animation, Sprite};

#[derive(Debug)]
pub struct AnimationPlayer<R> {
    backend: R,
    animation: Arc<Animation>,
    current_sprite: i16,
    frame: u32,
}

impl<R> AnimationPlayer<R> {
    #[inline]
    pub fn new(backend: R, animation: Arc<Animation>) -> Self {
        Self {
            backend,
            current_sprite: *animation.sprites.keys().next().unwrap(),
            animation,
            frame: 0,
        }
    }

    pub fn render(&mut self, initial: SpriteTransform)
    where
        R: Render,
    {
        let sprite = self.animation.sprites.get(&self.current_sprite).unwrap();
        self.backend
            .render_sprite(&self.animation, sprite, initial, self.frame);
        self.frame += 1;
    }

    #[inline]
    pub fn set_sprite(&mut self, sprite: i16) {
        self.current_sprite = sprite;
        self.frame = 0
    }

    #[inline]
    pub fn set_frame(&mut self, frame: u32) {
        self.frame = frame
    }

    #[inline]
    pub fn backend(&self) -> &R {
        &self.backend
    }

    #[inline]
    pub fn backend_mut(&mut self) -> &mut R {
        &mut self.backend
    }

    #[inline]
    pub fn animation(&self) -> Arc<Animation> {
        self.animation.clone()
    }

    #[inline]
    pub fn current_sprite(&self) -> &Sprite {
        self.animation.sprites.get(&self.current_sprite).unwrap()
    }

    #[inline]
    pub fn current_sprite_id(&self) -> i16 {
        self.current_sprite
    }
}
