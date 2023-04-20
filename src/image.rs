use std::collections::VecDeque;
use std::{cmp::Ordering, fs, str};

use crate::color::RGBColor;

#[derive(Debug)]
pub struct PPMImage {
    pub img_type: String,
    pub width: usize,
    pub height: usize,
    pub max_val: usize,
    pub data: Vec<RGBColor<i32>>,
}

#[derive(Debug)]
pub struct DWTImage {
    // approximation coefficients
    pub ll: Vec<RGBColor<i32>>,
    // vertical details
    pub lh: Vec<RGBColor<i32>>,
    // horizontal details
    pub hl: Vec<RGBColor<i32>>,
    // diagonal details
    pub hh: Vec<RGBColor<i32>>,
    pub orig_width: usize,
    pub orig_height: usize,
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
        let mut character = PPMImage::remove_whitespace(contents);

        while !character.is_whitespace() {
            buffer.push(character);
            character = contents.pop_front().unwrap() as char;
        }

        return vec_to_u32(&buffer).unwrap() as usize;
    }

    fn remove_whitespace(contents: &mut VecDeque<u8>) -> char {
        let mut character = contents.pop_front().unwrap_or_default() as char;
        while character.is_whitespace() {
            character = contents.pop_front().unwrap_or_default() as char;
        }
        return character;
    }

    fn remove_newline(contents: &mut VecDeque<u8>) -> char {
        let mut character = contents.pop_front().unwrap_or_default() as char;
        while character == '\n' || character == '\r' {
            character = contents.pop_front().unwrap_or_default() as char;
        }

        return character;
    }

    fn parse_next_color(contents: &mut VecDeque<u8>) -> RGBColor<i32> {
        let (r, g, b);

        r = PPMImage::remove_newline(contents) as i32;
        g = PPMImage::remove_newline(contents) as i32;
        b = PPMImage::remove_newline(contents) as i32;

        return RGBColor::new(r, g, b);
    }
}

impl DWTImage {
    pub fn new(
        ll: Vec<RGBColor<i32>>,
        lh: Vec<RGBColor<i32>>,
        hl: Vec<RGBColor<i32>>,
        hh: Vec<RGBColor<i32>>,
        orig_width: usize,
        orig_height: usize,
    ) -> DWTImage {
        DWTImage {
            ll,
            lh,
            hl,
            hh,
            orig_width,
            orig_height,
        }
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
                    vec_low[y * (img.width / 2) + x].sub(&vec_low[(y + 1) * (img.width / 2) + x]),
                );
                hh.push(
                    vec_high[y * (img.width / 2) + x].sub(&vec_high[(y + 1) * (img.width / 2) + x]),
                );
            }
        }

        return DWTImage::new(ll, lh, hl, hh, img.width, img.height);
    }

    fn horizontal_transform(img: &PPMImage) -> (Vec<RGBColor<i32>>, Vec<RGBColor<i32>>) {
        let mut vec_low = Vec::<RGBColor<i32>>::new();
        let mut vec_high = Vec::<RGBColor<i32>>::new();
        let range = if img.width % 2 == 0 {
            (0..img.width).step_by(2)
        } else {
            (0..img.width - 1).step_by(2)
        };
        for y in 0..img.height {
            for x in range.clone() {
                vec_low.push(img.data[y * img.width + x].add(&img.data[y * img.width + (x + 1)]));
                vec_high.push(img.data[y * img.width + x].sub(&img.data[y * img.width + (x + 1)]));
            }
        }

        return (vec_low, vec_high);
    }

    pub fn hide_message(self, mess: DWTImage) -> DWTImage {
        // blocking
        let (ia, ih, iv, id, sa) = (
            DWTImage::blocking_extract_one(self.ll, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(self.lh, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(self.hl, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(self.hh, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(mess.ll, mess.orig_width, mess.orig_height),
        );
        // matching
        let index = DWTImage::matching(&ia, &sa);

        println!("{:?}", index);
        todo!();
    }

    fn matching(ia: &Vec<[RGBColor<i32>; 4]>, sa: &Vec<[RGBColor<i32>; 4]>) -> (usize, usize) {
        let mut result: Vec<(f64, usize, usize)> = Vec::new();

        println!("{} {}", sa.len(), ia.len());
        for sa_index in 0..sa.len() {
            for ia_index in 0..ia.len() {
                result.push((DWTImage::root_mean_square_error(&sa[sa_index], &ia[ia_index]), sa_index, ia_index));
            }
        }

        quicksort::quicksort_by(
            &mut result,
            |e1: &(f64, usize, usize), e2: &(f64, usize, usize)| -> Ordering {
                if e1.0 < e2.0 {
                    return Ordering::Less;
                } else if e1.0 > e2.0 {
                    return Ordering::Greater;
                } else {
                    return Ordering::Equal;
                }
            },
        );

        return (result[0].1, result[0].2);
    }

    fn root_mean_square_error(vec1: &[RGBColor<i32>; 4], vec2: &[RGBColor<i32>; 4]) -> f64 {
        let mut result_color: RGBColor<f64> = RGBColor::new(0.0, 0.0, 0.0);

        for i in 0..4 {
            result_color.red += ((vec2[i].red - vec1[i].red) as f64).powf(2.0);
            result_color.green += ((vec2[i].green - vec1[i].green) as f64).powf(2.0);
            result_color.blue += ((vec2[i].blue - vec1[i].blue) as f64).powf(2.0);
        }

        result_color.red = (result_color.red / 4.0).sqrt();
        result_color.green = (result_color.green / 4.0).sqrt();
        result_color.blue = (result_color.blue / 4.0).sqrt();

        return (result_color.red + result_color.green + result_color.blue) / 3.0;
    }

    fn blocking_extract_one(
        mat: Vec<RGBColor<i32>>,
        orig_width: usize,
        orig_height: usize,
    ) -> Vec<[RGBColor<i32>; 4]> {
        let mut result: Vec<[RGBColor<i32>; 4]> = Vec::new();
        let mut temp_arr: [RGBColor<i32>; 4] = [RGBColor::new(0, 0, 0); 4];

        for y in (0..orig_height / 2 - 1).step_by(2) {
            for x in (0..orig_width / 2 - 1).step_by(2) {
                temp_arr[0] = mat[(y + 0) * (orig_width / 2) + (x + 0)];
                temp_arr[1] = mat[(y + 0) * (orig_width / 2) + (x + 1)];
                temp_arr[2] = mat[(y + 1) * (orig_width / 2) + (x + 0)];
                temp_arr[3] = mat[(y + 1) * (orig_width / 2) + (x + 1)];
                result.push(temp_arr);
            }
        }

        return result;
    }
}

pub fn vec_to_u32(digits: &Vec<char>) -> Option<u32> {
    const RADIX: u32 = 10;
    return digits
        .iter()
        .map(|c| c.to_digit(RADIX))
        .try_fold(0, |ans, i| i.map(|i| ans * RADIX + i));
}
