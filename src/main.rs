mod color;
mod image;
mod stegano;

extern crate quicksort;
use crate::stegano::DWTImage;

fn main() {
    let (key1, index_arr, width, height) =
        DWTImage::hide_image("./images/dog.ppm", "./images/banana.ppm").unwrap();
    DWTImage::extract_message_from_image(
        "./images/watermarked.ppm",
        "./images/extracted_img.ppm",
        key1,
        index_arr,
        width,
        height,
    )
    .unwrap();
}
