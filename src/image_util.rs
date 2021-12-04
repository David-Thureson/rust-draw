use image::{RgbImage, Rgb};
use crate::*;
use crate::grid::*;

pub fn main() {
    try_create_image_file();
}

pub fn save_grid<T, F>(grid: &Grid<T>, file_name: &str, color_func: &F, border_width: usize, border_color: Option<Color256>)
    where T: Clone,
          F: Fn(&T) -> Color256
{
    let border_color = color_rgb_to_rgb(&border_color.unwrap_or(Color256::black()));
    let border_add = border_width as u32 * 2;
    let (image_width, image_height) = (grid.width as u32 + border_add, grid.height as u32 + border_add);
    let mut img = RgbImage::new(image_width, image_height);

    // Fill in the image with the border color.
    if border_width > 0 {
        for image_y in 0..image_height {
            for image_x in 0..image_width {
                img.put_pixel(image_x, image_y, border_color);
            }
        }
    }

    for grid_y in 0..grid.height {
        for grid_x in 0..grid.width {
            let (image_x, image_y) = ((grid_x + border_width) as u32, (grid_y + border_width) as u32);
            let rgb = color_rgb_to_rgb(&color_func(&grid.get_xy(grid_x, grid_y)));
            img.put_pixel(image_x, image_y, rgb);
        }
    }

    img.save(file_name).unwrap();
}

fn color_rgb_to_rgb(color_rgb: &Color256) -> Rgb<u8> {
    Rgb([color_rgb.r, color_rgb.g, color_rgb.b])
}

// fn rectangle<T>(img: &mut ImageBuffer<T>, rgb: &Rgb<u8>, x1: u32, y1: u32, x2: u32, y2: u32) {
//     for y in
// }

fn try_create_image_file() {
    let mut img = RgbImage::new(32, 32);

    for x in 15..=17 {
        for y in 8..24 {
            img.put_pixel(x, y, Rgb([255, 0, 0]));
            img.put_pixel(y, x, Rgb([255, 0, 0]));
        }
    }

    img.save("Test.png").unwrap();
}



