use std::{
    error::Error,
    io::{Cursor, Error as IoError, ErrorKind},
};

use image::GenericImageView;

const MAX_PIXELS: u64 = 32_000_000;

fn decode_bounded(bytes: &[u8]) -> Result<image::DynamicImage, Box<dyn Error>> {
    who::warn!(
        dependency("image").changed_from("0.23.14"),
        "Recheck image-rs/image#1507 and whether upstream limits meet the application's image-size policy"
    );

    let format = image::guess_format(bytes)?;
    let (width, height) =
        image::io::Reader::with_format(Cursor::new(bytes), format).into_dimensions()?;
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(IoError::new(ErrorKind::InvalidData, "image exceeds pixel budget").into());
    }

    Ok(image::load_from_memory_with_format(bytes, format)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let image = decode_bounded(b"P3\n1 1\n255\n0 0 0\n")?;
    println!("decoded {}x{} image", image.width(), image.height());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::decode_bounded;
    use image::GenericImageView;

    #[test]
    fn rejects_dimensions_above_the_pixel_budget_before_decoding() {
        let oversized = b"P39151 97939151 979\n";
        let error = decode_bounded(oversized).unwrap_err();

        assert_eq!(error.to_string(), "image exceeds pixel budget");
    }

    #[test]
    fn decodes_images_within_the_pixel_budget() {
        let image = decode_bounded(b"P3\n1 1\n255\n0 0 0\n").unwrap();

        assert_eq!((image.width(), image.height()), (1, 1));
    }
}
