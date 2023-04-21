mod color;
mod image;

extern crate quicksort;
use crate::image::{DWTImage, PPMImage};

fn main() {
    let orginal_filepath = String::from("./images/test.ppm");
    let secret_filepath = String::from("./images/secret.ppm");

    let orginal_image = DWTImage::from_ppm(&PPMImage::from_file(&orginal_filepath));
    let message_image = DWTImage::from_ppm(&PPMImage::from_file(&secret_filepath));

    let watermarked_image = orginal_image.hide_message(message_image);
    watermarked_image.export_to_file("./images/watermarked.ppm");
}
