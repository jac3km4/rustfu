use std::path::PathBuf;

use euclid::default::Box2D;
use notan::app::{Color, Graphics};
use notan::draw::CreateDraw;
use rustfu_renderer::notan::NotanBackend;
use rustfu_renderer::player::AnimationPlayer;
use rustfu_renderer::render::{Measure, SpriteTransform};

const FRAME_PADDING: f32 = 96.;
const FRAME_TIME: i32 = 30;

trait FrameWriter {
    fn write_frame(&mut self, bytes: &[u8], frame: usize) -> anyhow::Result<()>;
}

impl FrameWriter for webp_animation::Encoder {
    fn write_frame(&mut self, bytes: &[u8], frame: usize) -> anyhow::Result<()> {
        self.add_frame(bytes, frame as i32 * FRAME_TIME)?;
        Ok(())
    }
}

pub fn write_webp(
    gfx: &mut Graphics,
    player: &mut AnimationPlayer<NotanBackend>,
    scale: f32,
) -> anyhow::Result<impl AsRef<[u8]>> {
    let (inner, outer) = calculate_dimensions(player, scale);
    let mut writer = webp_animation::Encoder::new((outer.width() as _, outer.height() as _))?;
    write_frames(gfx, player, &mut writer, scale, inner, outer)?;
    Ok(writer.finalize(player.current_sprite().frame_count() as i32 * FRAME_TIME)?)
}

#[derive(Debug)]
struct SplitPngFrames {
    dir: PathBuf,
    width: u32,
    height: u32,
}

impl FrameWriter for SplitPngFrames {
    fn write_frame(&mut self, bytes: &[u8], ts: usize) -> anyhow::Result<()> {
        let img = image::RgbaImage::from_raw(self.width, self.height, bytes.to_vec())
            .ok_or_else(|| anyhow::anyhow!("generated image was invalid"))?;
        img.save(self.dir.join(format!("frame_{}.png", ts)))?;
        Ok(())
    }
}

pub fn write_individual_frames(
    gfx: &mut Graphics,
    player: &mut AnimationPlayer<NotanBackend>,
    scale: f32,
    dir: PathBuf,
) -> anyhow::Result<()> {
    let (inner, outer) = calculate_dimensions(player, scale);
    let mut writer = SplitPngFrames {
        dir,
        width: outer.width() as _,
        height: outer.height() as _,
    };
    write_frames(gfx, player, &mut writer, scale, inner, outer)
}

fn write_frames(
    gfx: &mut Graphics,
    player: &mut AnimationPlayer<NotanBackend>,
    writer: &mut dyn FrameWriter,
    scale: f32,
    inner: Box2D<f32>,
    outer: Box2D<f32>,
) -> anyhow::Result<()> {
    let (output_x, output_y) = gfx.size();
    let output_ratio_x = output_x as f32 / outer.width();
    let output_ratio_y = output_y as f32 / outer.height();

    let target = gfx
        .create_render_texture(outer.width() as _, outer.height() as _)
        .build()
        .map_err(|err| anyhow::anyhow!("failed to create render texture: {}", err))?;

    let mut output = vec![0; outer.width() as usize * outer.height() as usize * 4];

    for i in 0..player.current_sprite().frame_count() {
        player.backend_mut().draw_mut().clear(Color::TRANSPARENT);

        let translation =
            SpriteTransform::translate((FRAME_PADDING - inner.min.x) * 2., FRAME_PADDING * 2.);
        let scale = SpriteTransform::scale(output_ratio_x * scale, -output_ratio_y * scale);
        player.render(scale.combine(&translation));
        gfx.render_to(&target, &player.backend_mut().swap(gfx.create_draw()));

        gfx.read_pixels(&target)
            .read_to(&mut output)
            .map_err(|err| anyhow::anyhow!("failed to read pixels: {}", err))?;
        writer.write_frame(&output, i)?;
    }
    Ok(())
}

fn calculate_dimensions(
    player: &AnimationPlayer<NotanBackend>,
    scale: f32,
) -> (Box2D<f32>, Box2D<f32>) {
    let scale = player.animation().scale() * scale;
    let inner = Measure::run(&player.animation(), player.current_sprite(), scale);
    (inner, inner.inflate(FRAME_PADDING, FRAME_PADDING))
}
