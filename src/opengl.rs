extern crate glium;
extern crate image;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use glium::glutin::event::Event;
use glium::glutin::event::StartCause;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event_loop::EventLoop;
#[cfg(unix)]
use glium::glutin::platform::unix::EventLoopExtUnix;
#[cfg(windows)]
use glium::glutin::platform::windows::EventLoopExtWindows;
use glium::texture::{RawImage2d, Texture2d};
use glium::{Blend, DrawParameters, Frame, IndexBuffer, Program, VertexBuffer};

use crate::render::{Render, SpriteTransform};
use crate::types::{Animation, Shape, Sprite};

use self::glium::{Display, Surface};
use glium::index::PrimitiveType;

pub struct RenderCommand {
    pub animation: Animation,
    pub image: image::RgbaImage,
    pub sprite: String,
}

pub fn run_renderer(receiver: Receiver<RenderCommand>) -> () {
    let events_loop: EventLoop<()> = EventLoop::new_any_thread();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(640.0, 640.0))
        .with_title("Renderer");
    let cb = glium::glutin::ContextBuilder::new();
    let display = Display::new(wb, cb, &events_loop).unwrap();

    let program = create_program(&display);

    let mut current_cmd: Option<RenderCommand> = None;
    let mut current_sprite: Option<Sprite> = None;
    let mut cache = HashMap::new();
    let mut texture = Texture2d::new(&display, vec![vec![(0u8, 0u8, 0u8, 0u8)]]).unwrap();
    let mut frame = 0;

    events_loop.run(move |event, _, control_flow| {
        let next_frame_time = Instant::now() + Duration::from_nanos(33_333_333);
        frame += 1;
        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => (),
                StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        if let Some(cmd) = &current_cmd {
            if let Some(sprite) = &current_sprite {
                let viewport = display.get_framebuffer_dimensions();
                let mut state = RenderState {
                    display: &display,
                    target: &mut target,
                    program: &program,
                    vbos: &mut cache,
                    texture: &texture,
                    viewport,
                };

                let scale = cmd
                    .borrow()
                    .animation
                    .index
                    .to_owned()
                    .and_then(|i| i.scale)
                    .unwrap_or(1.);
                state.render_sprite(&cmd.animation, sprite, SpriteTransform::scale(scale, scale), frame)
            }
        }

        if let Some(cmd) = receiver.try_recv().ok() {
            cache.clear();
            texture = load_texture(&display, cmd.image.clone());
            current_sprite = cmd
                .animation
                .sprites
                .values()
                .find(|sprite| sprite.name.name.as_ref() == Some(&cmd.sprite))
                .cloned();
            current_cmd = Some(cmd);
        }

        target.finish().unwrap();
    });
}

struct RenderState<'a> {
    pub display: &'a Display,
    pub target: &'a mut Frame,
    pub program: &'a Program,
    pub vbos: &'a mut HashMap<i16, VertexBuffer<Vertex>>,
    pub texture: &'a Texture2d,
    pub viewport: (u32, u32),
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

const BASE_SCALE: f32 = 4.;

impl<'a> Render for RenderState<'a> {
    fn render(&mut self, shape: &Shape, transformation: SpriteTransform) -> () {
        let display = &self.display;
        let vbo = self.vbos.entry(shape.id).or_insert_with(|| load_sprite(display, shape));
        let index_buffer =
            IndexBuffer::new(self.display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 2, 1, 3]).unwrap();

        let color = &transformation.color.color();

        let matrix_data: [[f32; 2]; 3] = transformation
            .position
            .post_scale(
                BASE_SCALE / self.viewport.0 as f32,
                -BASE_SCALE / self.viewport.1 as f32,
            )
            .to_row_arrays();
        let color_data: [f32; 4] = [color.red, color.green, color.blue, color.alpha];

        let uniforms = uniform! {
            matrix: [[matrix_data[0][0], matrix_data[0][1], 0.], [matrix_data[1][0], matrix_data[1][1], 0.], [matrix_data[2][0], matrix_data[2][1], 1.]],
            colors: color_data,
            tex: self.texture
        };

        let params = DrawParameters {
            blend: Blend {
                color: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::One,
                    destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
                },
                ..Blend::default()
            },
            ..DrawParameters::default()
        };

        self.target
            .draw(&*vbo, &index_buffer, &self.program, &uniforms, &params)
            .unwrap();
    }
}

fn create_program(display: &Display) -> Program {
    program!(
        display, 330 =>
        {
            vertex: r#"
                #version 330

                uniform mat3 matrix;

                in vec2 position;
                in vec2 tex_coords;

                out vec2 v_tex_coords;

                void main() {
                    gl_Position = vec4(matrix * vec3(position, 1), 1);
                    v_tex_coords = tex_coords;
                }
            "#,

            fragment: r#"
                #version 330

                uniform sampler2D tex;
                uniform vec4 colors;

                in vec2 v_tex_coords;
                
                out vec4 output;

                void main() {
                    output = texture(tex, v_tex_coords) * colors;
                }
            "#
        }
    )
    .unwrap()
}

fn load_sprite(display: &Display, shape: &Shape) -> VertexBuffer<Vertex> {
    let right = shape.offset_x + shape.width as f32;
    let top = shape.offset_y + shape.height as f32;
    let vertices = [
        Vertex {
            position: [shape.offset_x, shape.offset_y],
            tex_coords: [shape.left, -shape.bottom],
        },
        Vertex {
            position: [right, shape.offset_y],
            tex_coords: [shape.right, -shape.bottom],
        },
        Vertex {
            position: [shape.offset_x, top],
            tex_coords: [shape.left, -shape.top],
        },
        Vertex {
            position: [right, top],
            tex_coords: [shape.right, -shape.top],
        },
    ];
    VertexBuffer::new(display, &vertices).unwrap()
}

fn load_texture(display: &Display, image: image::RgbaImage) -> Texture2d {
    let dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);
    Texture2d::new(display, image).unwrap()
}
