mod image;
mod color;

use crate::image::{PPMImage, DWTImage};

fn main() {
    let orginal_image = DWTImage::from_ppm(&PPMImage::from_file("./images/test.ppm"));
    let message_image = DWTImage::from_ppm(&PPMImage::from_file("./images/test2.ppm"));

    println!("{:#?} {:#?}", orginal_image, message_image);
}
