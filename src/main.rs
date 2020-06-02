#![feature(option_zip)]
extern crate clear_coat;
extern crate iup_sys;
#[macro_use]
extern crate glium;

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;

use clear_coat::common_attrs_cbs::*;
use clear_coat::*;

use crate::location::Location;
use crate::opengl::{open_renderer, RenderCommand};
use crate::resources::Resources;
use crate::types::Animation;

pub mod decode;
pub mod frame_reader;
pub mod location;
pub mod opengl;
pub mod render;
pub mod resources;
pub mod types;

const LOCATION_FILE: &str = "location.txt";

fn main() {
    let location_file = File::open(Path::new(LOCATION_FILE))
        .and_then(|file| Location::read(&file))
        .ok();

    match location_file {
        None => show_initial_dialog().unwrap(),
        Some(location) => show_main_window(location).unwrap(),
    }
}

fn show_initial_dialog() -> io::Result<()> {
    let res = AlarmBuilder::new("Select path", "Please navigate to main Wakfu directory", "Find").popup();
    match res {
        1 => show_file_dialog(),
        _ => panic!("boom"),
    }
}

fn show_file_dialog() -> io::Result<()> {
    let dialog = FileDlg::new();
    dialog.set_dialog_type(FileDialogType::Dir);
    dialog.set_multiple_files(false);
    dialog
        .popup(ScreenPosition::CenterParent, ScreenPosition::CenterParent)
        .unwrap();

    match dialog.value_single() {
        None => Ok(()),
        Some(path) => {
            let mut file = File::create(Path::new(LOCATION_FILE))?;
            let location = Location(path);
            location.write(&mut file)?;
            show_main_window(location)?;
            Ok(())
        }
    }
}

fn show_main_window(location: Location) -> io::Result<()> {
    let mut resources = Resources::open(&location.0)?;
    let selected: Rc<RefCell<Option<(Animation, image::RgbaImage)>>> = Rc::new(RefCell::new(None));

    let (sender, receiver) = channel();

    thread::spawn(move || {
        open_renderer(receiver);
    });

    let anim_names = resources.npcs.list_animations().collect::<Vec<_>>();

    let anm_list = Rc::new(List::new());
    anm_list.set_multiple(false);
    anm_list.set_max_size(180, 480);
    anm_list.set_min_size(180, 480);
    anm_list.set_items(anim_names);

    let record_list = Rc::new(List::new());
    record_list.set_multiple(false);
    record_list.set_max_size(240, 480);
    record_list.set_min_size(240, 480);

    let button = Button::with_title("Play");

    let selected_ref = selected.clone();
    let record_list_ref = record_list.clone();

    record_list.action_event().add(move |args: &ListActionArgs| {
        match (*selected_ref.borrow()).clone() {
            None => (),
            Some((animation, image)) => {
                let sprite = args.text.to_owned();
                sender
                    .send(RenderCommand {
                        animation,
                        image,
                        sprite,
                    })
                    .unwrap();
            }
        };
    });

    anm_list.action_event().add(move |args: &ListActionArgs| {
        let animation = resources.npcs.load_animation(args.text).unwrap();
        let texture_id = animation.texture.clone().unwrap().name;
        let image = resources.npcs.load_texture(&texture_id).unwrap();
        let records = animation
            .sprites
            .iter()
            .filter_map(|(_, sprite)| sprite.name.name.clone())
            .collect::<Vec<_>>();
        let mut reference = selected.borrow_mut();
        *reference = Some((animation, image));
        record_list_ref.set_items(records);
    });

    let dialog = Dialog::with_child(hbox!(anm_list.clone(), record_list, button).set_top_level_margin_and_gap());

    dialog.set_title("Animation Browser");
    dialog.popup(ScreenPosition::Center, ScreenPosition::Center).unwrap();
    Ok(())
}
