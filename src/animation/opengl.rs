extern crate glium;
extern crate image;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use glium::glutin::event::{Event, StartCause, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
#[cfg(unix)]
use glium::glutin::platform::unix::EventLoopExtUnix;
#[cfg(windows)]
use glium::glutin::platform::windows::EventLoopExtWindows;
use glium::texture::{RawImage2d, Texture2d};
use glium::{Blend, DrawParameters, IndexBuffer, Program, VertexBuffer};

use crate::animation::render::{Render, SpriteTransform};
use crate::animation::types::{Animation, Shape, Sprite};
use euclid::Transform2D;

use self::glium::{Display, Surface};
use glium::backend::Facade;
use glium::index::PrimitiveType;

pub struct RenderCommand {
    pub animation: Animation,
    pub image: image::RgbaImage,
    pub sprite: String,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

struct RenderState<'a, F, S> {
    pub display: &'a F,
    pub target: &'a mut S,
    pub program: &'a Program,
    pub vbos: &'a mut HashMap<i16, VertexBuffer<Vertex>>,
    pub texture: &'a Texture2d,
    pub viewport: (u32, u32),
}

const BASE_SCALE: f32 = 4.;

impl<'a, F: Facade, S: Surface> Render for RenderState<'a, F, S> {
    fn render(&mut self, shape: &Shape, transformation: SpriteTransform) -> () {
        let display = self.display;
        let vbo = self.vbos.entry(shape.id).or_insert_with(|| load_sprite(display, shape));
        let ebo = IndexBuffer::new(self.display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 2, 1, 3]).unwrap();

        let color = transformation.color.color();

        let matrix = transformation
            .position
            .post_transform(&viewport_transform(self.viewport))
            .to_row_arrays();

        let uniforms = uniform! {
            matrix: [[matrix[0][0], matrix[0][1], 0.], [matrix[1][0], matrix[1][1], 0.], [matrix[2][0], matrix[2][1], 1.]],
            colors: [color.red, color.green, color.blue, color.alpha],
            tex: self.texture
        };

        self.target
            .draw(&*vbo, &ebo, &self.program, &uniforms, &draw_parameters())
            .unwrap();
    }
}

fn draw_parameters<'b>() -> DrawParameters<'b> {
    DrawParameters {
        blend: Blend {
            color: glium::BlendingFunction::Addition {
                source: glium::LinearBlendingFactor::One,
                destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
            },
            alpha: glium::BlendingFunction::Addition {
                source: glium::LinearBlendingFactor::One,
                destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
            },
            ..Blend::default()
        },
        ..DrawParameters::default()
    }
}

fn viewport_transform(viewport: (u32, u32)) -> Transform2D<f32, (), ()> {
    Transform2D::create_scale(BASE_SCALE / viewport.0 as f32, BASE_SCALE / viewport.1 as f32)
}

fn load_sprite<F: Facade>(display: &F, shape: &Shape) -> VertexBuffer<Vertex> {
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

pub fn create_texture<F: Facade>(display: &F, image: image::RgbaImage) -> Texture2d {
    let dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);
    Texture2d::new(display, image).unwrap()
}

pub fn create_program<F: Facade>(display: &F) -> Program {
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

pub fn draw<F: Facade, S: Surface>(
    display: &F,
    target: &mut S,
    program: &Program,
    vbos: &mut HashMap<i16, VertexBuffer<Vertex>>,
    texture: &Texture2d,
    animation: &Animation,
    sprite: &Sprite,
    frame: u32,
) -> () {
    let mut state = RenderState {
        display,
        target,
        program,
        vbos,
        texture,
        viewport: (640, 640),
    };
    let scale = animation.index.clone().and_then(|i| i.scale).unwrap_or(1.);
    state.render_sprite(&animation, sprite, SpriteTransform::scale(scale, scale), frame)
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

        if let (Some(cmd), Some(sprite)) = (&current_cmd, &current_sprite) {
            draw(
                &display,
                &mut target,
                &program,
                &mut cache,
                &texture,
                &cmd.animation,
                sprite,
                frame,
            );
        }

        if let Some(cmd) = receiver.try_recv().ok() {
            cache.clear();
            texture = create_texture(&display, cmd.image.clone());
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
