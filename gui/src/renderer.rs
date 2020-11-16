use quicksilver::geom::Vector;
use quicksilver::graphics::{Color, Graphics, Image, PixelFormat, Surface};
use quicksilver::input::Event;
use quicksilver::{Input, Settings, Timer, Window};
use rustfu_renderer::backend::QuicksilverBackend;
use rustfu_renderer::types::{Animation, Sprite};
use std::fs::File;
use std::sync::mpsc::Receiver;

pub enum RenderCommand {
    Draw(Animation, image::RgbaImage, String),
    SaveGif,
}

pub fn run_renderer(receiver: Receiver<RenderCommand>) -> Result<(), String> {
    let settings = Settings {
        title: "Renderer",
        size: Vector::new(320., 320.),
        resizable: true,
        ..Settings::default()
    };
    quicksilver::run(settings, |window, gfx, input| app(window, gfx, input, receiver));
}

async fn app(
    window: Window,
    gfx: Graphics,
    mut input: Input,
    receiver: Receiver<RenderCommand>,
) -> quicksilver::Result<()> {
    let mut timer = Timer::time_per_second(30.);
    let mut frame = 0;
    let mut backend = QuicksilverBackend::new(gfx, &window);
    let mut selected_sprite = None;
    let mut selected_animation = None;
    loop {
        while let Some(event) = input.next_event().await {
            match event {
                Event::Resized(size_event) => {
                    backend.context().set_camera_size(size_event.size());
                    backend.set_viewport(size_event.size());
                }
                _ => {}
            }
        }
        match receiver.try_recv().ok() {
            Some(RenderCommand::Draw(animation, image, sprite)) => {
                selected_sprite = animation
                    .sprites
                    .values()
                    .find(|spr| spr.name.name.as_ref() == Some(&sprite))
                    .cloned();
                selected_animation = Some(animation);
                backend.set_atlas(image);
            }
            Some(RenderCommand::SaveGif) => {
                if let (Some(sprite), Some(animation)) = (&selected_sprite, &selected_animation) {
                    save_gif(&mut backend, animation, sprite, window.size())?
                }
            }
            None => {}
        }

        if timer.exhaust().is_some() {
            backend.context().clear(Color::BLACK);
            if let (Some(sprite), Some(animation)) = (&selected_sprite, &selected_animation) {
                backend.render(animation, sprite, frame);
            }
            backend.context().present(&window)?;
            frame += 1;
        }
    }
}

fn save_gif(
    backend: &mut QuicksilverBackend,
    animation: &Animation,
    sprite: &Sprite,
    size: Vector,
) -> quicksilver::Result<()> {
    let width = size.x as u16;
    let height = size.y as u16;

    let attachment = Image::from_raw(backend.context(), None, width.into(), height.into(), PixelFormat::RGBA)?;
    let surface = Surface::new(backend.context(), attachment)?;

    let file_path = format!("saved/{}.gif", sprite.name.name_crc);
    std::fs::create_dir_all("saved").expect("Failed to create the screenshot directory");
    let mut encoder = gif::Encoder::new(File::create(file_path).unwrap(), width, height, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    for frame in 0..sprite.frame_count() {
        backend.context().clear(Color::BLACK.with_alpha(0.));
        backend.render(animation, sprite, frame as u32);
        backend.context().flush_surface(&surface)?;
        let mut data = surface.screenshot(backend.context(), 0, 0, width.into(), height.into(), PixelFormat::RGBA);

        let mut frame = gif::Frame::from_rgba_speed(width, height, &mut data, 10);
        frame.delay = 3;
        frame.dispose = gif::DisposalMethod::Background;
        encoder.write_frame(&frame).expect("Failed to encode GIF frame");
    }
    Ok(())
}
