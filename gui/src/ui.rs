use std::sync::Arc;

use notan::egui;
use rustfu_renderer::types::Animation;

use crate::resources::{AnimatedEntityKind, AnimationEntry};

#[derive(Debug)]
pub struct UiState {
    npcs: Vec<AnimationEntry>,
    interactives: Vec<AnimationEntry>,
    pets: Vec<AnimationEntry>,

    animation: Option<Arc<Animation>>,
    selected_entity: AnimatedEntityKind,
    filter: String,
    filtered_entries: Option<Vec<usize>>,
    error: Option<String>,
    available_space: egui::Rect,

    events: Vec<UiEvent>,
}

impl UiState {
    pub fn new(
        npcs: Vec<AnimationEntry>,
        interactives: Vec<AnimationEntry>,
        pets: Vec<AnimationEntry>,
    ) -> Self {
        Self {
            npcs,
            interactives,
            pets,
            animation: None,
            selected_entity: AnimatedEntityKind::Monster,
            filter: String::new(),
            filtered_entries: None,
            error: None,
            available_space: egui::Rect::ZERO,
            events: Vec::new(),
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_top_bar(ui);

            ui.horizontal_top(|ui| {
                self.draw_anim_list(ui);
                self.draw_sprite_list(ui);

                self.available_space = ui.available_rect_before_wrap();
            });
        });
    }

    fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("Tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                for item in [
                    AnimatedEntityKind::Monster,
                    AnimatedEntityKind::InteractiveElementModel,
                    AnimatedEntityKind::Pet,
                ] {
                    if ui
                        .selectable_label(self.selected_entity == item, item.label())
                        .clicked()
                    {
                        self.selected_entity = item;
                        self.filtered_entries = None;
                    }
                }

                ui.separator();

                if ui.button("Save as WEBP").clicked() {
                    self.events.push(UiEvent::SaveAsWebp)
                }
                if ui.button("Save as Frames").clicked() {
                    self.events.push(UiEvent::SaveAsFrames)
                }

                ui.separator();

                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
            });
            ui.add_space(4.);
        });
    }

    fn draw_anim_list(&mut self, ui: &mut egui::Ui) {
        let text_style = egui::TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);

        egui::SidePanel::new(egui::panel::Side::Left, "AnimationList")
            .exact_width(200.)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(4.);

                let edit = egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter")
                    .show(ui);

                if edit.response.changed() {
                    self.filtered_entries = None;
                }

                ui.separator();

                let entries = match self.selected_entity {
                    AnimatedEntityKind::Monster => &self.npcs,
                    AnimatedEntityKind::InteractiveElementModel => &self.interactives,
                    AnimatedEntityKind::Pet => &self.pets,
                };

                let filtered = self.filtered_entries.get_or_insert_with(|| {
                    entries
                        .iter()
                        .enumerate()
                        .filter(|(_, entry)| entry.label().contains(self.filter.as_str()))
                        .map(|(i, _)| i)
                        .collect()
                });

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, filtered.len(), |ui, row_range| {
                        for &idx in &filtered[row_range.start..row_range.end] {
                            let entry = &entries[idx];
                            if ui.selectable_label(false, entry.label()).clicked() {
                                self.events.push(UiEvent::RequestSprite(entry.id()))
                            }
                        }
                    });
            });
    }

    fn draw_sprite_list(&mut self, ui: &mut egui::Ui) {
        if let Some(animation) = &self.animation {
            egui::SidePanel::new(egui::panel::Side::Left, "SpriteList")
                .exact_width(200.)
                .resizable(false)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (&id, sprite) in &animation.sprites {
                                let Some(name) = sprite.name.name.as_deref() else {
                                    continue;
                                };
                                if ui.selectable_label(false, name).clicked() {
                                    self.events.push(UiEvent::SetSprite(id))
                                }
                            }
                        });
                });
        }
    }

    #[inline]
    pub fn available_space(&self) -> egui::Rect {
        self.available_space
    }

    #[inline]
    pub fn selected_entity(&self) -> AnimatedEntityKind {
        self.selected_entity
    }

    #[inline]
    pub fn set_animation(&mut self, animation: Arc<Animation>) {
        self.animation = Some(animation);
    }

    #[inline]
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    #[inline]
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    #[inline]
    pub fn take_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.events)
    }
}

#[derive(Debug)]
pub enum UiEvent {
    RequestSprite(i32),
    SetSprite(i16),
    SaveAsWebp,
    SaveAsFrames,
}
