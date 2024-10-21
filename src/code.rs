use std::{io::Cursor, sync::Arc};

use barcoders::{generators::image::Image, sym::code128::Code128};
use image::{GenericImage, ImageBuffer, Rgba};

type ImageOutput = ImageBuffer<Rgba<u8>, Vec<u8>>;

const HEIGHT: u32 = 30;

pub fn into_barcode(code: &str) -> String {
    format!("\u{00C0}{}", code.to_ascii_uppercase())
}

fn generate_code_128(code: &str) -> anyhow::Result<ImageOutput> {
    let code = Code128::new(code)?;
    let middleware = Image::png(HEIGHT);
    let encoded = code.encode();
    let ret = middleware.generate_buffer(&encoded[..])?;
    Ok(ret)
}

pub fn merge2(self_id: Arc<String>, code: &str) -> anyhow::Result<ImageOutput> {
    let image1 = generate_code_128(&self_id)?;
    let image2 = generate_code_128(code)?;

    /* ImageOutput::new(
        std::cmp::max(image1.width(), image2.width()),
        HEIGHT * 2 + 50,
    ); */

    let mut empty = ImageOutput::from_pixel(
        std::cmp::max(image1.width(), image2.width()) + 40,
        HEIGHT * 2 + 40,
        Rgba([255, 255, 255, 255]),
    );

    //log::debug!("{} {}", image1.height(), image1.width());
    //log::debug!("{} {}", image2.height(), image2.width());

    //image1.copy_from(&image2, 0, image1.height())?;
    empty.copy_from(&image1, 20, 10)?;
    empty.copy_from(&image2, 20, image1.height() + 30)?;

    Ok(image::imageops::rotate270(&empty))
}

pub fn merge2memory(self_id: Arc<String>, code: &str) -> anyhow::Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    merge2(self_id, code)?.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}
