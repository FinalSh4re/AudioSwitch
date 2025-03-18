use std::io::Cursor;

use anyhow::{Context, Result, anyhow};
use image::{GenericImageView, Rgba, RgbaImage};
use tray_icon::Icon;

pub type HexColor = String;

pub fn generate_icon(color: HexColor) -> Result<Icon> {
    // Load the source image (which may have transparency)
    let img_bytes = include_bytes!("../assets/app.png");
    let mut image = image::ImageReader::new(Cursor::new(img_bytes));
    image.set_format(image::ImageFormat::Png);
    let img = image.decode().expect("Failed to load image.");

    let (width, height) = img.dimensions();
    let (r, g, b) = hex_to_rgb(color)?;

    let background_color = Rgba([r, g, b, 255]);

    let mut background = RgbaImage::from_pixel(width, height, background_color);

    let overlay = img.to_rgba8();

    // Composite overlay image onto the background by alpha blending.
    for (x, y, overlay_pixel) in overlay.enumerate_pixels() {
        let bg_pixel = background.get_pixel(x, y);

        // Normalize alpha value to [0.0, 1.0]
        let alpha = overlay_pixel.0[3] as f32 / 255.0;

        // Blend each channel: result = fg * alpha + bg * (1 - alpha)
        let blended = [
            (overlay_pixel.0[0] as f32 * alpha + bg_pixel.0[0] as f32 * (1.0 - alpha)).round()
                as u8,
            (overlay_pixel.0[1] as f32 * alpha + bg_pixel.0[1] as f32 * (1.0 - alpha)).round()
                as u8,
            (overlay_pixel.0[2] as f32 * alpha + bg_pixel.0[2] as f32 * (1.0 - alpha)).round()
                as u8,
            255,
        ];
        background.put_pixel(x, y, Rgba(blended));
    }

    let (icon_width, icon_height) = background.dimensions();
    let icon_rgba = background.into_raw();

    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).context("Failed to create Icon.")
}

fn hex_to_rgb(hex: HexColor) -> Result<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(anyhow!("Hex color must be 6 digits long."));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok((r, g, b))
}
