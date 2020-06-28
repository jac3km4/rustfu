#![windows_subsystem = "windows"]
#[macro_use]
extern crate glium;
extern crate image;

use crate::data::resources::Resources;
use glium::glutin;
use glium::glutin::event_loop::EventLoop;
use wakfudecrypt::document::Document;
use wakfudecrypt::types::monster::Monster;
pub mod animation;
pub mod data;
use crate::animation::opengl::*;
use glium::texture::{MipmapsOption, RawImage2d, Texture2d, UncompressedFloatFormat};
use glium::Surface;
use std::borrow::Borrow;
use std::collections::HashMap;

pub fn main() {
    let event_loop = EventLoop::new();

    let ctx = glutin::ContextBuilder::new()
        .build_headless(&event_loop, glium::glutin::dpi::PhysicalSize::new(640, 640))
        .unwrap();

    let display = glium::HeadlessRenderer::new(ctx).unwrap();

    let surface = Texture2d::empty_with_format(
        &display,
        UncompressedFloatFormat::F32F32F32F32,
        MipmapsOption::EmptyMipmaps,
        640,
        640,
    )
    .unwrap();
    let mut target = surface.as_surface();

    let program = create_program(&display);

    let mut resources = Resources::open(&std::env::current_dir().unwrap()).unwrap();
    let monsters: Document<Monster> = resources.load_data().unwrap();
    for monster in monsters.elements {
        if let Ok(animation) = resources.npc_animations.load_animation(&format!("{}", monster.gfx)) {
            let static_sprite = animation
                .borrow()
                .sprites
                .values()
                .find(|s| s.name.name == Some("1_AnimStatique".to_owned()));
            if let (Some(texture), Some(sprite)) = (&animation.texture, static_sprite) {
                let image = resources.npc_animations.load_texture(&texture.name).unwrap();
                let texture = create_texture(&display, image);
                draw(
                    &display,
                    &mut target,
                    &program,
                    &mut HashMap::new(),
                    &texture,
                    &animation,
                    sprite,
                    0,
                );

                let mut rendered: RawImage2d<u8> = surface.read();
                target.clear_color(0.0, 0.0, 0.0, 0.0);
                let result =
                    image::RgbaImage::from_raw(rendered.width, rendered.height, rendered.data.to_mut().clone())
                        .unwrap();
                result.save(format!("./renders/{}.png", monster.id)).unwrap();
            }
        }
    }
}
