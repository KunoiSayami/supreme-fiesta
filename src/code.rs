use std::{io::Cursor, sync::Arc};

use barcoders::{generators::image::Image, sym::code128::Code128};
use image::{GenericImage, ImageBuffer, Rgba};

type ImageOutput = ImageBuffer<Rgba<u8>, Vec<u8>>;

const HEIGHT: u32 = 30;

pub fn into_barcode(code: &str) -> String {
    format!("\u{00C0}{}", code.to_ascii_uppercase())
}

fn get_number_location(code: &str) -> Option<(usize, Option<usize>)> {
    let mut start = None;
    for (index, char) in code.chars().enumerate() {
        if char.is_numeric() && start.is_none() {
            start.replace(index);
        } else if !char.is_numeric() && start.is_some() {
            return Some((start.unwrap(), Some(index)));
        }
    }
    if let Some(start) = start {
        return Some((start, None));
    }
    None
}

#[allow(unused)]
pub fn specify_barcode(code: &str) -> String {
    let Some((index, end)) = get_number_location(code) else {
        return into_barcode(code);
    };

    if end.is_none() {
        format!("\u{0106}{code}")
    } else {
        let mut code = code.to_ascii_uppercase();
        code.insert(end.unwrap(), '\u{00C0}');
        code.insert(index, '\u{0106}');
        //code.insert_str(index, "@$\u{0106}");
        format!("\u{00C0}{code}")
    }
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

    let mut base = ImageOutput::from_pixel(
        std::cmp::max(image1.width(), image2.width()) + 40,
        HEIGHT * 2 + 40,
        Rgba([255, 255, 255, 255]),
    );

    //log::debug!("{} {}", image1.height(), image1.width());
    //log::debug!("{} {}", image2.height(), image2.width());

    //image1.copy_from(&image2, 0, image1.height())?;
    base.copy_from(&image1, 20, 10)?;
    base.copy_from(&image2, 20, image1.height() + 30)?;

    Ok(image::imageops::rotate270(&base))
}

pub fn merge2memory(self_id: Arc<String>, code: &str) -> anyhow::Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    merge2(self_id, code)?.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}
