use image::{DynamicImage, GenericImageView, GrayImage, Rgb, RgbImage};
use imageproc::contrast::equalize_histogram_mut;

pub fn equalize_color_histogram(image: &DynamicImage) -> Option<RgbImage> {
    let pixel_count = image.pixels().count();
    let mut red_channel = Vec::with_capacity(pixel_count);
    let mut green_channel = Vec::with_capacity(pixel_count);
    let mut blue_channel = Vec::with_capacity(pixel_count);
    for (_, _, pixel) in image.pixels() {
        red_channel.push(pixel[0]);
        green_channel.push(pixel[1]);
        blue_channel.push(pixel[2]);
    }
    let mut red_channel_grayscale =
        GrayImage::from_vec(image.width(), image.height(), red_channel)?;
    let mut green_channel_grayscale =
        GrayImage::from_vec(image.width(), image.height(), green_channel)?;
    let mut blue_channel_grayscale =
        GrayImage::from_vec(image.width(), image.height(), blue_channel)?;

    equalize_histogram_mut(&mut red_channel_grayscale);
    equalize_histogram_mut(&mut green_channel_grayscale);
    equalize_histogram_mut(&mut blue_channel_grayscale);

    let mut reconstructed = RgbImage::new(image.width(), image.height());
    for y in 0..image.height() {
        for x in 0..image.width() {
            reconstructed.put_pixel(
                x,
                y,
                Rgb::from([
                    red_channel_grayscale[(x, y)].0[0],
                    green_channel_grayscale[(x, y)].0[0],
                    blue_channel_grayscale[(x, y)].0[0],
                ]),
            );
        }
    }

    Some(reconstructed)
}
