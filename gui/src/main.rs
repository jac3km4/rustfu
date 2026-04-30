#![windows_subsystem = "windows"]

use app::AppState;
use rfd::{FileDialog, MessageDialog};
use notan::draw::DrawConfig;
use notan::egui::*;
use notan::prelude::*;
use resources::Resources;

mod app;
mod resources;
mod translations;
mod ui;
mod writer;

#[notan_main]
fn main() {
    MessageDialog::new()
        .set_title("Rustfu")
        .set_description("Select the Wakfu installation folder")
        .set_buttons(rfd::MessageButtons::Ok)
        .show();

    let Some(path) = FileDialog::new().pick_folder() else {
        return;
    };

    let result: anyhow::Result<_> = (|| {
        let resources = Resources::open(path)?;
        let state = AppState::new(resources)?;
        Ok(state)
    })();

    let state = match result {
        Ok(state) => state,
        Err(err) => {
            let msg = format!("Could not load resources at the specified path: {}", err);
            MessageDialog::new()
                .set_title("Error")
                .set_description(&msg)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            return;
        }
    };

    let win = WindowConfig::new()
        .set_vsync(true)
        .set_lazy_loop(true)
        .set_high_dpi(true)
        .set_resizable(true)
        .set_lazy_loop(false)
        .set_size(1024, 768)
        .set_title("Rustfu");

    notan::init_with(|_: &mut Assets, _: &mut Graphics| state)
        .add_config(win)
        .add_config(EguiConfig)
        .add_config(DrawConfig)
        .draw(draw)
        .build()
        .unwrap();
}

fn draw(_app: &mut App, gfx: &mut Graphics, plugins: &mut Plugins, state: &mut AppState) {
    if !state.should_render() {
        return;
    }

    let ui_draw = plugins.egui(|ctx| state.update_ui(ctx));
    gfx.render(&ui_draw);

    let canvas_draw = state.update_canvas(gfx);
    gfx.render(&canvas_draw);
}
