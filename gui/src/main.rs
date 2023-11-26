#![windows_subsystem = "windows"]

use app::AppState;
use native_dialog::{FileDialog, MessageDialog};
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
    if MessageDialog::new()
        .set_text("Select the Wakfu installation folder")
        .show_alert()
        .is_err()
    {
        return;
    }
    let Ok(Some(path)) = FileDialog::new().show_open_single_dir() else {
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
            MessageDialog::new().set_text(&msg).show_alert().unwrap();
            return;
        }
    };

    let win = WindowConfig::new()
        .set_vsync(true)
        .set_lazy_loop(true)
        .set_high_dpi(true)
        .set_resizable(true)
        .set_lazy_loop(false)
        .set_size(1024, 768);

    notan::init_with(|_: &mut Assets, _: &mut Graphics| state)
        .add_config(win)
        .add_config(EguiConfig)
        .add_config(DrawConfig)
        .draw(draw)
        .build()
        .unwrap()
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
