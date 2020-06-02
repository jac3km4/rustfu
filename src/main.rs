#![windows_subsystem = "windows"]
#[macro_use]
extern crate glium;
extern crate image;

use std::fs::File;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::opengl::{run_renderer, RenderCommand};
use crate::resources::{AnimationArchive, Resources};
use crate::types::Animation;
use iced::*;
use image::RgbaImage;
use std::borrow::{Borrow, BorrowMut};

pub mod decode;
pub mod frame_reader;
pub mod opengl;
pub mod render;
pub mod resources;
pub mod types;

pub fn main() {
    let resources = Resources::open(&std::env::current_dir().unwrap()).unwrap();
    let (sender, receiver) = channel();

    thread::spawn(move || run_renderer(receiver));
    State::run(Settings::with_flags((resources, sender)));
}

#[derive(Debug, Clone)]
enum AnimationType {
    Npc = 0,
    Interactive = 1,
    Pet = 2,
}

impl AnimationType {
    pub fn from(i: usize) -> Option<AnimationType> {
        match i {
            0 => Some(AnimationType::Npc),
            1 => Some(AnimationType::Interactive),
            2 => Some(AnimationType::Pet),
            _ => None,
        }
    }
}

struct State {
    input: text_input::State,
    type_list: SelectList,
    animation_list: SelectList,
    record_list: SelectList,
    filter: String,
    selected_type: AnimationType,
    resources: Resources<File>,
    sender: Sender<RenderCommand>,
    animation: Option<Animation>,
    image: Option<RgbaImage>,
}

#[derive(Debug, Clone)]
enum Message {
    Reload,
    FilterChanged(String),
    TypeSelected(AnimationType),
    AnimationSelected(usize),
    RecordSelected(usize),
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = (Resources<File>, Sender<RenderCommand>);

    fn new(flags: Self::Flags) -> (State, Command<Message>) {
        let mut initial = State {
            input: text_input::State::default(),
            type_list: SelectList::new(vec!["Npc".to_owned(), "Interactive".to_owned(), "Pet".to_owned()]),
            animation_list: SelectList::new(vec![]),
            record_list: SelectList::new(vec![]),
            filter: "".to_owned(),
            selected_type: AnimationType::Npc,
            resources: flags.0,
            sender: flags.1,
            animation: None,
            image: None,
        };
        let cmd = initial.borrow_mut().update(Message::Reload);
        (initial, cmd)
    }

    fn title(&self) -> String {
        "Animation Browser".to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Reload => {
                let filter = &self.filter.clone();
                let animations = self
                    .selected_archive()
                    .list_animations()
                    .filter(|str| str.contains(filter))
                    .map(|s| s.to_owned())
                    .collect();
                self.animation_list.set_options(animations);
            }
            Message::FilterChanged(filter) => {
                self.filter = filter;
                self.update(Message::Reload);
            }
            Message::TypeSelected(t) => {
                self.selected_type = t;
                self.update(Message::Reload);
            }
            Message::AnimationSelected(i) => {
                let archive = match self.selected_type {
                    AnimationType::Npc => &mut self.resources.npcs,
                    AnimationType::Interactive => &mut self.resources.interactives,
                    AnimationType::Pet => &mut self.resources.pets,
                };
                let name = self.animation_list.options.get(i).unwrap();
                let animation = archive.load_animation(name).unwrap();
                let texture_id = animation.texture.clone().unwrap().name;
                let options = animation
                    .borrow()
                    .sprites
                    .iter()
                    .filter_map(|(_, sprite)| sprite.name.name.clone())
                    .collect::<Vec<_>>();

                self.image = Some(archive.load_texture(&texture_id).unwrap());
                self.animation = Some(animation);
                self.record_list.set_options(options);
                self.animation_list.update(SelectMessage(i))
            }
            Message::RecordSelected(i) => {
                let State {
                    animation,
                    image,
                    record_list,
                    ..
                } = self;
                if let Some(animation) = animation {
                    if let Some(image) = image {
                        let sprite = record_list.options.get(i).unwrap().to_owned();
                        let cmd = RenderCommand {
                            animation: animation.clone(),
                            image: image.clone(),
                            sprite,
                        };
                        self.sender.send(cmd).unwrap();
                        self.record_list.update(SelectMessage(i));
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let State {
            input,
            filter,
            type_list,
            animation_list,
            record_list,
            ..
        } = self;
        let input = TextInput::new(input, "Filter by ID", filter, Message::FilterChanged).padding(5);

        let types = type_list
            .view()
            .map(|m| Message::TypeSelected(AnimationType::from(m.0).unwrap()));
        let animations = animation_list.view().map(|m| Message::AnimationSelected(m.0));
        let records = record_list.view().map(|m| Message::RecordSelected(m.0));

        let content = Row::new()
            .push(Container::new(types).width(Length::Fill).style(theme::SelectList))
            .push(Container::new(animations).width(Length::Fill).style(theme::SelectList))
            .push(Container::new(records).width(Length::Fill).style(theme::SelectList))
            .spacing(20);

        Container::new(
            Column::new()
                .push(input)
                .push(Space::with_height(Length::Units(20)))
                .push(content)
                .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl State {
    fn selected_archive(&mut self) -> &mut AnimationArchive<File> {
        match self.selected_type {
            AnimationType::Npc => &mut self.resources.npcs,
            AnimationType::Interactive => &mut self.resources.interactives,
            AnimationType::Pet => &mut self.resources.pets,
        }
    }
}

struct SelectList {
    scroll: scrollable::State,
    buttons: Vec<button::State>,
    options: Vec<String>,
    selected: usize,
}

#[derive(Debug, Clone)]
struct SelectMessage(usize);

impl SelectList {
    fn new(options: Vec<String>) -> Self {
        SelectList {
            scroll: scrollable::State::default(),
            buttons: SelectList::default_buttons(options.len()),
            options,
            selected: 0,
        }
    }

    fn update(&mut self, message: SelectMessage) {
        self.selected = message.0;
    }

    fn set_options(&mut self, options: Vec<String>) {
        self.buttons = SelectList::default_buttons(options.len());
        self.options = options;
    }

    fn view(&mut self) -> Element<SelectMessage> {
        let SelectList {
            buttons,
            scroll,
            options,
            ..
        } = self;
        let body =
            buttons
                .iter_mut()
                .zip(options)
                .enumerate()
                .fold(Column::new().padding(2), |col, (i, (state, option))| {
                    let button = Button::new(state, Text::new(option.to_owned()))
                        .on_press(SelectMessage(i))
                        .width(Length::Fill)
                        .style(theme::SelectOption);
                    col.push(button)
                });

        Scrollable::new(scroll).push(body.width(Length::Fill)).into()
    }

    fn default_buttons(size: usize) -> Vec<button::State> {
        std::iter::repeat(button::State::default()).take(size).collect()
    }
}

mod theme {
    use iced::*;

    pub struct SelectOption;

    impl button::StyleSheet for SelectOption {
        fn active(&self) -> button::Style {
            button::Style {
                background: None,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(Color::from_rgb8(200, 200, 200))),
                ..button::Style::default()
            }
        }
    }

    pub struct SelectList;

    impl container::StyleSheet for SelectList {
        fn style(&self) -> container::Style {
            container::Style {
                border_width: 1,
                border_color: Color::from_rgb8(200, 200, 200),
                border_radius: 4,
                ..container::Style::default()
            }
        }
    }
}
