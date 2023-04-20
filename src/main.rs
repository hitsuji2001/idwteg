mod color;
mod image;

extern crate quicksort;
use crate::image::{DWTImage, PPMImage};

fn main() {
    let orginal_image = DWTImage::from_ppm(&PPMImage::from_file("./images/test.ppm"));
    let message_image = DWTImage::from_ppm(&PPMImage::from_file("./images/secret.ppm"));

    orginal_image.hide_message(message_image);
}
