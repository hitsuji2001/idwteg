use std::collections::VecDeque;
use std::{fs, str};

use crate::color::{RGBColor, RGBColorFlexer};

#[derive(Debug)]
pub struct PPMImage {
    pub img_type: String,
    pub width: usize,
    pub height: usize,
    pub max_val: usize,
    pub data: Vec<RGBColor>,
}

#[derive(Debug)]
pub struct DWTImage {
    pub ll: Vec<RGBColorFlexer>,
    pub lh: Vec<RGBColorFlexer>,
    pub hl: Vec<RGBColorFlexer>,
    pub hh: Vec<RGBColorFlexer>,
}

impl PPMImage {
    pub fn new() -> Self {
        return PPMImage {
            img_type: String::new(),
            width: 0,
            height: 0,
            max_val: 0,
            data: Vec::new(),
        };
    }

    pub fn from_file(file_path: &str) -> PPMImage {
        let mut img = PPMImage::new();
        let mut contents = VecDeque::from(fs::read(file_path).expect("Could not open file"));

        // first 2 bytes will always is in {P1 -> P6}
        img.img_type = vec![
            contents.pop_front().unwrap() as char,
            contents.pop_front().unwrap() as char,
        ]
        .iter()
        .collect();

        img.width = PPMImage::parse_next_normal_number_from_header(&mut contents);
        img.height = PPMImage::parse_next_normal_number_from_header(&mut contents);
        img.max_val = PPMImage::parse_next_normal_number_from_header(&mut contents);

        for _ in 0..img.width * img.height {
            let color = PPMImage::parse_next_color(&mut contents);
            img.data.push(color);
        }

        return img;
    }

    fn parse_next_normal_number_from_header(contents: &mut VecDeque<u8>) -> usize {
        let mut buffer: Vec<char> = Vec::new();
        let mut character;

        character = contents.pop_front().unwrap() as char;
        while character.is_whitespace() {
            character = contents.pop_front().unwrap() as char;
        }
        while !character.is_whitespace() {
            buffer.push(character);
            character = contents.pop_front().unwrap() as char;
        }

        return vec_to_u32(&buffer).unwrap() as usize;
    }

    fn parse_next_color(contents: &mut VecDeque<u8>) -> RGBColor {
        return RGBColor::new(
            contents.pop_front().unwrap_or_default(),
            contents.pop_front().unwrap_or_default(),
            contents.pop_front().unwrap_or_default(),
        );
    }
}

impl DWTImage {
    pub fn new(
        ll: Vec<RGBColorFlexer>,
        lh: Vec<RGBColorFlexer>,
        hl: Vec<RGBColorFlexer>,
        hh: Vec<RGBColorFlexer>,
    ) -> DWTImage {
        DWTImage { ll, lh, hl, hh }
    }

    pub fn from_ppm(img: &PPMImage) -> DWTImage {
        let (mut ll, mut lh, mut hl, mut hh) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (vec_low, vec_high) = DWTImage::horizontal_transform(img);

        let range = if img.height % 2 == 0 {
            (0..img.height).step_by(2)
        } else {
            (0..img.height - 1).step_by(2)
        };

        // vertical transform
        for y in range.clone() {
            for x in 0..img.width / 2 {
                ll.push(
                    vec_low[(y + 1) * (img.width / 2) + x].add(&vec_low[y * (img.width / 2) + x]),
                );
                hl.push(
                    vec_high[(y + 1) * (img.width / 2) + x].add(&vec_high[y * (img.width / 2) + x]),
                );

                lh.push(
                    vec_low[y * (img.width / 2) + x]
                        .subtract(&vec_low[(y + 1) * (img.width / 2) + x]),
                );
                hh.push(
                    vec_high[y * (img.width / 2) + x]
                        .subtract(&vec_high[(y + 1) * (img.width / 2) + x]),
                );
            }
        }

        return DWTImage::new(ll, lh, hl, hh);
    }

    fn horizontal_transform(img: &PPMImage) -> (Vec<RGBColorFlexer>, Vec<RGBColorFlexer>) {
        let mut vec_low = Vec::new();
        let mut vec_high = Vec::new();

        let range = if img.width % 2 == 0 {
            (0..img.width).step_by(2)
        } else {
            (0..img.width - 1).step_by(2)
        };
        for y in 0..img.height {
            for x in range.clone() {
                vec_low.push(img.data[y * img.width + x].add(&img.data[y * img.width + (x + 1)]));
                vec_high
                    .push(img.data[y * img.width + x].subtract(&img.data[y * img.width + (x + 1)]));
            }
        }

        return (vec_low, vec_high);
    }
}

pub fn vec_to_u32(digits: &Vec<char>) -> Option<u32> {
    const RADIX: u32 = 10;
    return digits
        .iter()
        .map(|c| c.to_digit(RADIX))
        .try_fold(0, |ans, i| i.map(|i| ans * RADIX + i));
}
