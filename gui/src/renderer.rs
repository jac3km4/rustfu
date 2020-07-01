use rustfu_renderer::gl::{DefaultLocations, Program, RenderState, Texture, SpriteVertex};
use rustfu_renderer::types::{Animation, Sprite};
use std::collections::HashMap;
use std::rc::Rc;

use glow::HasContext;
use glutin::dpi::PhysicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

#[cfg(unix)]
use glutin::platform::unix::EventLoopExtUnix;
#[cfg(windows)]
use glutin::platform::windows::EventLoopExtWindows;

pub struct RenderCommand {
    pub animation: Animation,
    pub image: image::RgbaImage,
    pub sprite: String,
}

pub fn run_renderer(receiver: Receiver<RenderCommand>) -> Result<(), String> {
    let event_loop: EventLoop<()> = EventLoop::new_any_thread();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Renderer")
        .with_inner_size(glutin::dpi::LogicalSize::new(640, 640));
    let windowed_context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .expect("Could not create a window");
    let windowed_context = unsafe { windowed_context.make_current().expect("Failed to bind GL context") };
    let context = Rc::new(glow::Context::from_loader_function(|s| {
        windowed_context.get_proc_address(s) as *const _
    }));
    let program = Program::default(context.clone())?;
    let locations = DefaultLocations::from(&program).ok_or("Failed to fetch locations")?;
    let gl = context.clone();

    let mut current_animation: Option<Animation> = None;
    let mut current_sprite: Option<Sprite> = None;
    let mut current_texture = None;
    let mut vertexes: HashMap<i16, SpriteVertex<glow::Context>> = HashMap::new();
    let mut frame = 0;

    event_loop.run(move |event, _, control_flow| {
        let next_frame_time = Instant::now() + Duration::from_nanos(33_333_333);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        match event {
            Event::LoopDestroyed => return,
            Event::MainEventsCleared => windowed_context.window().request_redraw(),
            Event::RedrawRequested(_) => unsafe {
                frame += 1;
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.clear_color(0f32, 0f32, 0f32, 0f32);

                if let (Some(animation), Some(sprite), Some(texture)) =
                    (&current_animation, &current_sprite, &current_texture)
                {
                    let PhysicalSize { width, height } = windowed_context.window().inner_size();
                    let mut state = RenderState::new(gl.clone(), &mut vertexes, texture, &locations, (width, height));
                    state.render(animation, sprite, frame);
                }

                windowed_context.swap_buffers().unwrap();

                if let Some(command) = receiver.try_recv().ok() {
                    let RenderCommand {
                        sprite,
                        animation,
                        image,
                    } = command;
                    vertexes.clear();
                    current_texture = Some(Texture::new(gl.clone(), image).expect("Could not load texture"));
                    current_sprite = animation
                        .sprites
                        .values()
                        .find(|spr| spr.name.name.as_ref() == Some(&sprite))
                        .cloned();
                    current_animation = Some(animation);
                }
            },
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(*physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }
    });
}
