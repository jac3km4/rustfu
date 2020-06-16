#![windows_subsystem = "windows"]
#[macro_use]
extern crate glium;
extern crate image;

use std::fmt::Display;
use std::fs::File;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::animation::opengl::{run_renderer, RenderCommand};
use crate::animation::types::Animation;
use crate::data::resources::Resources;
use crate::data::translations::Translation;
use iced::*;
use image::RgbaImage;
use std::borrow::{Borrow, BorrowMut};
use wakfudecrypt::types::interactive_element_model::InteractiveElementModel;
use wakfudecrypt::types::monster::Monster;
use wakfudecrypt::types::pet::Pet;

pub mod animation;
pub mod data;

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
    type_list: SelectList<String>,
    animation_list: SelectList<NamedOption<String>>,
    record_list: SelectList<String>,
    filter: String,
    selected_type: AnimationType,
    options: Vec<NamedOption<String>>,
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
            options: vec![],
            resources: flags.0,
            sender: flags.1,
            animation: None,
            image: None,
        };
        let cmd = initial.borrow_mut().update(Message::TypeSelected(AnimationType::Npc));
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
                    .options
                    .iter()
                    .filter(|opt| opt.0.contains(filter))
                    .cloned()
                    .collect();
                self.animation_list.set_options(animations);
            }
            Message::FilterChanged(filter) => {
                self.filter = filter;
                self.update(Message::Reload);
            }
            Message::TypeSelected(t) => {
                self.selected_type = t;
                match self.selected_type {
                    AnimationType::Npc => {
                        self.options = self
                            .resources
                            .load_data::<Monster>()
                            .unwrap()
                            .elements
                            .iter()
                            .map(|m| {
                                let name = self
                                    .resources
                                    .translations
                                    .get(Translation::Monster, &format!("{}", m.id));
                                NamedOption(
                                    name.cloned().unwrap_or(format!("Monster {}", m.id)),
                                    format!("{}", m.gfx),
                                )
                            })
                            .collect();
                    }
                    AnimationType::Interactive => {
                        self.options = self
                            .resources
                            .load_data::<InteractiveElementModel>()
                            .unwrap()
                            .elements
                            .iter()
                            .map(|e| {
                                let name = self
                                    .resources
                                    .translations
                                    .get(Translation::InteractiveElementView, &format!("{}", e.id));
                                NamedOption(name.cloned().unwrap_or(format!("IE {}", e.id)), format!("{}", e.gfx))
                            })
                            .collect();
                    }
                    AnimationType::Pet => {
                        self.options = self
                            .resources
                            .load_data::<Pet>()
                            .unwrap()
                            .elements
                            .iter()
                            .map(|p| {
                                let name = self
                                    .resources
                                    .translations
                                    .get(Translation::Item, &format!("{}", p.item_ref_id));
                                NamedOption(
                                    name.cloned().unwrap_or(format!("Pet {}", p.id)),
                                    format!("{}", p.gfx_id),
                                )
                            })
                            .collect();
                    }
                }
                self.update(Message::Reload);
            }
            Message::AnimationSelected(i) => {
                let archive = match self.selected_type {
                    AnimationType::Npc => &mut self.resources.npc_animations,
                    AnimationType::Interactive => &mut self.resources.interactive_animations,
                    AnimationType::Pet => &mut self.resources.pet_animations,
                };
                let name = self.animation_list.options.get(i).unwrap();
                if let Result::Ok(animation) = archive.load_animation(&name.1) {
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

#[derive(Debug, Clone)]
struct NamedOption<A>(String, A);

impl<A: Display> Display for NamedOption<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct SelectList<A> {
    scroll: scrollable::State,
    buttons: Vec<button::State>,
    options: Vec<A>,
    selected: usize,
}

#[derive(Debug, Clone)]
struct SelectMessage(usize);

impl<A: Display> SelectList<A> {
    fn new(options: Vec<A>) -> Self {
        SelectList {
            scroll: scrollable::State::default(),
            buttons: SelectList::<A>::default_buttons(options.len()),
            options,
            selected: 0,
        }
    }

    fn update(&mut self, message: SelectMessage) {
        self.selected = message.0;
    }

    fn set_options(&mut self, options: Vec<A>) {
        self.buttons = SelectList::<A>::default_buttons(options.len());
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
                    let button = Button::new(state, Text::new(format!("{}", option)))
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
