extern crate glium;
extern crate image;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use glium::glutin::event::Event;
use glium::glutin::event::StartCause;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::platform::windows::EventLoopExtWindows;
use glium::texture::{RawImage2d, Texture2d};
use glium::{Blend, DrawParameters, Frame, IndexBuffer, Program, VertexBuffer};

use crate::renderer::Renderer;
use crate::types::{Animation, Shape, Transformation};

use self::glium::{Display, Surface};
use glium::index::PrimitiveType;

pub struct RenderCommand {
    pub animation: Animation,
    pub image: image::RgbaImage,
    pub sprite: String,
}

struct RenderState<'a> {
    pub display: &'a Display,
    pub target: &'a mut Frame,
    pub program: &'a Program,
    pub vbos: &'a mut HashMap<i16, VertexBuffer<Vertex>>,
    pub texture: &'a Texture2d,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub fn open_renderer(receiver: Receiver<RenderCommand>) -> () {
    let events_loop: EventLoop<()> = EventLoop::new_any_thread();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(768.0, 768.0))
        .with_title("Renderer");
    let cb = glium::glutin::ContextBuilder::new();
    let display = Display::new(wb, cb, &events_loop).unwrap();

    let program = create_program(&display);

    let mut current_cmd: Option<RenderCommand> = None;
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
            draw(&display, &mut target, &program, &mut cache, &texture, &cmd, frame);
        }

        if let Some(cmd) = receiver.try_recv().ok() {
            cache.clear();
            texture = load_texture(&display, cmd.image.clone());
            current_cmd = Some(cmd);
        }

        target.finish().unwrap();
    });
}

fn draw(
    display: &Display,
    target: &mut Frame,
    program: &Program,
    vbos: &mut HashMap<i16, VertexBuffer<Vertex>>,
    texture: &Texture2d,
    command: &RenderCommand,
    frame: u32,
) {
    if let Some(sprite) = command
        .animation
        .sprites
        .values()
        .find(|sprite| sprite.name.name.as_ref() == Some(&command.sprite))
    {
        let mut state = RenderState {
            display,
            target,
            program,
            vbos,
            texture,
        };

        state.draw_sprite(&command.animation, sprite, vec![], frame)
    }
}

impl<'a> Renderer for RenderState<'a> {
    fn render(&mut self, shape: &Shape, transformation: Transformation) -> () {
        let display = &self.display;
        let vbo = self.vbos.entry(shape.id).or_insert_with(|| load_sprite(display, shape));
        let index_buffer =
            IndexBuffer::new(self.display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 2, 1, 3]).unwrap();

        let color = &transformation.color;
        let matrix: [[f32; 2]; 3] = transformation.position.to_row_arrays();
        let colors: [f32; 4] = [color.red, color.green, color.blue, color.alpha];
        let uniforms = uniform! {
            matrix: [[matrix[0][0], matrix[0][1], 0.], [matrix[1][0], matrix[1][1], 0.], [matrix[2][0], matrix[2][1], 1.]],
            colors: colors,
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
    let vertices = [
        Vertex {
            position: [0f32, 0f32],
            tex_coords: [shape.left, -shape.bottom],
        },
        Vertex {
            position: [1f32, 0f32],
            tex_coords: [shape.right, -shape.bottom],
        },
        Vertex {
            position: [0f32, 1f32],
            tex_coords: [shape.left, -shape.top],
        },
        Vertex {
            position: [1f32, 1f32],
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
