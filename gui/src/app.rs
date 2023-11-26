use std::fs::File;
use std::sync::Arc;
use std::time::{Duration, Instant};

use native_dialog::FileDialog;
use notan::draw::{CreateDraw, Draw};
use notan::egui;
use notan::prelude::*;
use rustfu_renderer::notan::NotanBackend;
use rustfu_renderer::player::AnimationPlayer;
use rustfu_renderer::render::{Measure, SpriteTransform};
use rustfu_renderer::types::Animation;
use wakfudecrypt::types::interactive_element_model::InteractiveElementModel;
use wakfudecrypt::types::monster::Monster;
use wakfudecrypt::types::pet::Pet;

use crate::resources::{AnimatedEntityKind, AnimationEntry, Resources};
use crate::ui::{UiEvent, UiState};
use crate::writer;

const DEFAULT_SCALE: f32 = 2.;
const FRAME_TIME: u64 = 30;

#[derive(notan::AppState)]
pub struct AppState {
    ui: UiState,
    player: Option<AnimationPlayer<NotanBackend>>,
    last_render: Instant,

    io_requests: ringbuf::HeapProducer<SpriteRequest>,
    io_receiver: Option<oneshot::Receiver<anyhow::Result<SpriteResponse>>>,
}

impl AppState {
    pub fn new(mut resources: Resources<File>) -> anyhow::Result<Self> {
        let npcs = AnimationEntry::load_all::<_, Monster>(&mut resources)?;
        let interactives = AnimationEntry::load_all::<_, InteractiveElementModel>(&mut resources)?;
        let pets = AnimationEntry::load_all::<_, Pet>(&mut resources)?;
        let (producer, consumer) = ringbuf::HeapRb::<SpriteRequest>::new(10).split();

        std::thread::spawn(move || Self::io_handler(consumer, &mut resources));

        Ok(Self {
            ui: UiState::new(npcs, interactives, pets),
            player: None,
            last_render: Instant::now(),
            io_requests: producer,
            io_receiver: None,
        })
    }

    fn draw(&mut self, gfx: &mut Graphics) -> Draw {
        if let Some(player) = &mut self.player {
            let scale = player.animation().scale() * DEFAULT_SCALE;
            let sprite_box = Measure::run(&player.animation(), player.current_sprite(), scale);
            let available_box = self.ui.available_space();

            let position = available_box.left_top() + available_box.size() / 2.
                - egui::Vec2::from(sprite_box.min.to_array())
                - egui::Vec2::from(sprite_box.size().to_array()) / 2.;

            let transform = SpriteTransform::scale(scale, scale)
                .combine(&SpriteTransform::translate(position.x, position.y));

            player.render(transform);
            let result = player.backend_mut().swap(gfx.create_draw());

            self.last_render = Instant::now();
            result
        } else {
            gfx.create_draw()
        }
    }

    fn handle_events(&mut self, gfx: &mut Graphics) {
        if let Some(receiver) = &mut self.io_receiver {
            if let Ok(resp) = receiver.try_recv() {
                let Some(SpriteResponse { animation, texture }) = self.unwrap_result(resp) else {
                    return;
                };
                let tex = gfx
                    .create_texture()
                    .from_bytes(texture.as_raw(), texture.width(), texture.height())
                    .with_filter(TextureFilter::Linear, TextureFilter::Linear)
                    .build()
                    .map_err(|err| anyhow::anyhow!("could not create texture: {}", err));
                let Some(tex) = self.unwrap_result(tex) else {
                    return;
                };

                let animation = Arc::new(animation);
                let backend = NotanBackend::new(gfx.create_draw(), tex);
                let player = AnimationPlayer::new(backend, animation.clone());

                self.ui.set_animation(animation);
                self.player = Some(player);
                self.io_receiver = None;
            }
        }

        let events = self.ui.take_events();
        if events.is_empty() {
            return;
        }
        self.ui.clear_error();

        for event in events {
            match event {
                UiEvent::RequestSprite(id) => {
                    let (sender, receiver) = oneshot::channel();
                    let kind = self.ui.selected_entity();
                    self.io_requests
                        .push(SpriteRequest { id, kind, sender })
                        .unwrap();
                    self.io_receiver = Some(receiver);
                }
                UiEvent::SetSprite(id) => {
                    if let Some(player) = &mut self.player {
                        player.set_sprite(id);
                    }
                }
                UiEvent::SaveAsWebp => {
                    if let Some(player) = &mut self.player {
                        let backend = player.backend().clone_with_draw(gfx.create_draw());
                        let mut tmp = AnimationPlayer::new(backend, player.animation());
                        tmp.set_sprite(player.current_sprite_id());

                        let result = (|| {
                            let Some(path) = FileDialog::new()
                                .set_filename("output.webp")
                                .show_save_single_file()?
                            else {
                                return Ok(());
                            };

                            let result = writer::write_webp(gfx, &mut tmp, DEFAULT_SCALE)?;
                            std::fs::write(path, result)?;
                            Ok(())
                        })();
                        self.unwrap_result(result);
                    }
                }
                UiEvent::SaveAsFrames => {
                    if let Some(player) = &mut self.player {
                        let backend = player.backend().clone_with_draw(gfx.create_draw());
                        let mut tmp = AnimationPlayer::new(backend, player.animation());
                        tmp.set_sprite(player.current_sprite_id());

                        let result = (|| {
                            let Some(dir) = FileDialog::new().show_open_single_dir()? else {
                                return Ok(());
                            };
                            writer::write_individual_frames(gfx, &mut tmp, DEFAULT_SCALE, dir)
                        })();
                        self.unwrap_result(result);
                    }
                }
            }
        }
    }

    fn io_handler(
        mut consumer: ringbuf::HeapConsumer<SpriteRequest>,
        resources: &mut Resources<File>,
    ) {
        loop {
            while let Some(req) = consumer.pop() {
                let source = match req.kind {
                    AnimatedEntityKind::Monster => &mut resources.npc_animations,
                    AnimatedEntityKind::InteractiveElementModel => {
                        &mut resources.interactive_animations
                    }
                    AnimatedEntityKind::Pet => &mut resources.pet_animations,
                };

                let res = (|| {
                    let animation = source.load_animation(&req.id.to_string())?;
                    let texture = animation
                        .texture
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("animation {} has no texture", req.id))?;
                    let texture = source.load_texture(&texture.name.to_string())?;
                    Ok(SpriteResponse::new(animation, texture))
                })();

                req.sender.send(res).ok();
            }
            std::thread::sleep(Duration::from_millis(FRAME_TIME));
        }
    }

    #[inline]
    pub fn should_render(&self) -> bool {
        self.last_render.elapsed() >= Duration::from_millis(FRAME_TIME)
    }

    #[inline]
    pub fn update_ui(&mut self, ctx: &egui::Context) {
        self.ui.draw(ctx);
    }

    #[inline]
    pub fn update_canvas(&mut self, gfx: &mut Graphics) -> Draw {
        self.handle_events(gfx);
        self.draw(gfx)
    }

    fn unwrap_result<A>(&mut self, result: anyhow::Result<A>) -> Option<A> {
        match result {
            Ok(value) => Some(value),
            Err(err) => {
                self.ui.set_error(err.to_string());
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct SpriteRequest {
    id: i32,
    kind: AnimatedEntityKind,
    sender: oneshot::Sender<anyhow::Result<SpriteResponse>>,
}

#[derive(Debug)]
pub struct SpriteResponse {
    animation: Animation,
    texture: image::RgbaImage,
}

impl SpriteResponse {
    #[inline]
    pub fn new(animation: Animation, texture: image::RgbaImage) -> Self {
        Self { animation, texture }
    }
}
