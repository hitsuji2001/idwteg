use std::collections::VecDeque;
use std::{cmp::Ordering, fs, io::Write, str};

use crate::color::RGBColor;

type Block<T> = [RGBColor<T>; 4];
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

    // FIXME: This version currently will not parse comment
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

    pub fn export_to_file(self, file_path: &str) -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)?;
        let mut buffer: Vec<u8> = Vec::new();

        writeln!(&mut file, "{}", self.img_type)?;
        writeln!(&mut file, "{} {}", self.width, self.height)?;
        writeln!(&mut file, "{}", self.max_val)?;

        for value in self.data {
            buffer.push(value.red as u8);
            buffer.push(value.green as u8);
            buffer.push(value.blue as u8);
        }
        file.write_all(&buffer)?;

        Ok(())
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

    pub fn hide_message(&self, mess: DWTImage) -> PPMImage {
        // blocking
        let (ia, mut ih, mut iv, mut id, sa) = (
            DWTImage::blocking_extract_one(&self.ll, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&self.lh, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&self.hl, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&self.hh, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&mess.ll, mess.orig_width, mess.orig_height),
        );
        // matching
        let index = DWTImage::matching(&sa, &ia);
        // block differences computation
        let mut bdc: Block<i32> = [RGBColor::new(0, 0, 0); 4];
        for i in 0..4 {
            bdc[i].red = sa[0][i].red - ia[index][i].red;
            bdc[i].green = sa[0][i].green - ia[index][i].green;
            bdc[i].blue = sa[0][i].blue - ia[index][i].blue;
        }
        // block replacement
        DWTImage::block_replacement(&bdc, &mut ih, &mut iv, &mut id);

        let watermarked_image =
            DWTImage::rearrange_blocks(&ia, &ih, &iv, &id, self.orig_width, self.orig_height);

        return watermarked_image.inverse_dwt();
    }

    fn inverse_dwt(&self) -> PPMImage {
        let mut result_image: PPMImage = PPMImage::new();
        let (x_left, x_right, y_left, y_right) = self.inverse_vertical_transform();
        let secret_image = DWTImage::inverse_horizontal_transform(x_left, x_right, y_left, y_right);
        let mut block_count = 0;

        result_image.img_type = String::from("P6");
        result_image.width = self.orig_width;
        result_image.height = self.orig_height;
        result_image.max_val = 255;
        result_image.data = vec![RGBColor::new(0, 0, 0); self.orig_width * self.orig_height];

        for y in (0..self.orig_height).step_by(2) {
            for x in (0..self.orig_width).step_by(2) {
                result_image.data[(y + 0) * self.orig_width + (x + 0)] =
                    secret_image[block_count][0];
                result_image.data[(y + 0) * self.orig_width + (x + 1)] =
                    secret_image[block_count][1];
                result_image.data[(y + 1) * self.orig_width + (x + 0)] =
                    secret_image[block_count][2];
                result_image.data[(y + 1) * self.orig_width + (x + 1)] =
                    secret_image[block_count][3];
                block_count += 1;
            }
        }

        return result_image;
    }

    fn rearrange_blocks(
        ia: &Vec<Block<i32>>,
        ih: &Vec<Block<i32>>,
        iv: &Vec<Block<i32>>,
        id: &Vec<Block<i32>>,
        width: usize,
        height: usize,
    ) -> DWTImage {
        let (ll, lh, hl, hh) = (
            DWTImage::rearrange_one_block(&ia, width, height),
            DWTImage::rearrange_one_block(&ih, width, height),
            DWTImage::rearrange_one_block(&iv, width, height),
            DWTImage::rearrange_one_block(&id, width, height),
        );

        return DWTImage::new(ll, lh, hl, hh, width, height);
    }

    fn rearrange_one_block(
        arr: &Vec<Block<i32>>,
        width: usize,
        height: usize,
    ) -> Vec<RGBColor<i32>> {
        let mut result: Vec<RGBColor<i32>> =
            vec![RGBColor::new(0, 0, 0); (width / 2) * (height / 2)];
        let mut block_count = 0;

        for y in (0..height / 2).step_by(2) {
            for x in (0..width / 2).step_by(2) {
                result[(y + 0) * (width / 2) + (x + 0)] = arr[block_count][0];
                result[(y + 0) * (width / 2) + (x + 1)] = arr[block_count][1];
                result[(y + 1) * (width / 2) + (x + 0)] = arr[block_count][2];
                result[(y + 1) * (width / 2) + (x + 1)] = arr[block_count][3];
                block_count += 1;
            }
        }

        return result;
    }

    fn block_replacement(
        bd: &Block<i32>,
        ih: &mut Vec<Block<i32>>,
        iv: &mut Vec<Block<i32>>,
        id: &mut Vec<Block<i32>>,
    ) {
        let (ih_index, iv_index, id_index) = (
            DWTImage::find_most_fit_block_index(&bd, &ih),
            DWTImage::find_most_fit_block_index(&bd, &iv),
            DWTImage::find_most_fit_block_index(&bd, &id),
        );
        let elements = vec![ih[ih_index], iv[iv_index], id[id_index]];

        let mut result: Vec<(f64, usize)> = Vec::new();
        result.push((DWTImage::root_mean_square_error(bd, &elements[0]), ih_index));
        result.push((DWTImage::root_mean_square_error(bd, &elements[1]), iv_index));
        result.push((DWTImage::root_mean_square_error(bd, &elements[2]), id_index));

        quicksort::quicksort_by(&mut result, DWTImage::float_usize_tuple_compare);
        match result[0].1 {
            x if x == ih_index => {
                ih[ih_index] = *bd;
            }
            x if x == iv_index => {
                iv[iv_index] = *bd;
            }
            x if x == id_index => {
                id[id_index] = *bd;
            }
            _ => {
                eprintln!("Unreachable, index = {}", result[0].1);
            }
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

    fn float_usize_tuple_compare(e1: &(f64, usize), e2: &(f64, usize)) -> Ordering {
        if e1.0 < e2.0 {
            return Ordering::Less;
        } else if e1.0 > e2.0 {
            return Ordering::Greater;
        } else {
            return Ordering::Equal;
        }
    }

    fn find_most_fit_block_index(bdc: &Block<i32>, arr: &Vec<Block<i32>>) -> usize {
        let mut result: Vec<(f64, usize)> = Vec::new();
        for i in 0..arr.len() {
            result.push((DWTImage::root_mean_square_error(&bdc, &arr[i]), i));
        }

        quicksort::quicksort_by(&mut result, DWTImage::float_usize_tuple_compare);
        return result[0].1;
    }

    #[allow(dead_code)]
    fn block_differences_computation(
        sa: &Vec<Block<i32>>,
        ia: &Vec<Block<i32>>,
    ) -> Vec<Block<i32>> {
        let mut result: Vec<Block<i32>> = Vec::new();
        let mut temp_arr: Block<i32> = [RGBColor::new(0, 0, 0); 4];
        for sa_index in 0..sa.len() {
            for ia_index in 0..ia.len() {
                for i in 0..4 {
                    temp_arr[i] = RGBColor::new(
                        sa[sa_index][i].red - ia[ia_index][i].red,
                        sa[sa_index][i].green - ia[ia_index][i].green,
                        sa[sa_index][i].blue - ia[ia_index][i].blue,
                    );
                }
                result.push(temp_arr);
            }
        }
        return result;
    }

    fn root_mean_square_error(vec1: &Block<i32>, vec2: &Block<i32>) -> f64 {
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
        mat: &Vec<RGBColor<i32>>,
        orig_width: usize,
        orig_height: usize,
    ) -> Vec<Block<i32>> {
        let mut result: Vec<Block<i32>> = Vec::new();
        let mut temp_arr: Block<i32> = [RGBColor::new(0, 0, 0); 4];

        for y in (0..orig_height / 2).step_by(2) {
            for x in (0..orig_width / 2).step_by(2) {
                temp_arr[0] = mat[(y + 0) * (orig_width / 2) + (x + 0)];
                temp_arr[1] = mat[(y + 0) * (orig_width / 2) + (x + 1)];
                temp_arr[2] = mat[(y + 1) * (orig_width / 2) + (x + 0)];
                temp_arr[3] = mat[(y + 1) * (orig_width / 2) + (x + 1)];
                result.push(temp_arr);
            }
        }

        return result;
    }

    fn matching(sa: &Vec<Block<i32>>, ia: &Vec<Block<i32>>) -> usize {
        let mut result: Vec<(f64, usize)> = Vec::new();
        for ia_index in 0..ia.len() {
            result.push((
                DWTImage::root_mean_square_error(&sa[0], &ia[ia_index]),
                ia_index,
            ));
        }
        quicksort::quicksort_by(&mut result, DWTImage::float_usize_tuple_compare);
        return result[0].1;
    }

    fn inverse_horizontal_transform(
        x_left: Vec<RGBColor<i32>>,
        x_right: Vec<RGBColor<i32>>,
        y_left: Vec<RGBColor<i32>>,
        y_right: Vec<RGBColor<i32>>,
    ) -> Vec<Block<i32>> {
        let mut result: Vec<Block<i32>> = Vec::new();
        let mut x: RGBColor<i32>;
        let mut y: RGBColor<i32>;
        let mut temp_block: Block<i32> = [RGBColor::new(0, 0, 0); 4];

        for i in 0..x_left.len() {
            x = (x_left[i].add(&x_right[i])).div_by(2);
            y = x_left[i].sub(&x);
            temp_block[0] = x;
            temp_block[1] = y;
            x = (y_left[i].add(&y_right[i])).div_by(2);
            y = y_left[i].sub(&x);
            temp_block[2] = x;
            temp_block[3] = y;
            result.push(temp_block);
        }

        return result;
    }

    fn inverse_vertical_transform(
        &self,
    ) -> (
        Vec<RGBColor<i32>>,
        Vec<RGBColor<i32>>,
        Vec<RGBColor<i32>>,
        Vec<RGBColor<i32>>,
    ) {
        let mut x_left: Vec<RGBColor<i32>> = Vec::new();
        let mut x_right: Vec<RGBColor<i32>> = Vec::new();
        let mut y_left: Vec<RGBColor<i32>> = Vec::new();
        let mut y_right: Vec<RGBColor<i32>> = Vec::new();
        let mut x: RGBColor<i32>;
        let mut y: RGBColor<i32>;

        for i in 0..self.ll.len() {
            x = (self.ll[i].add(&self.lh[i])).div_by(2);
            y = self.ll[i].sub(&x);

            x_left.push(x);
            y_left.push(y);

            x = (self.hl[i].add(&self.hh[i])).div_by(2);
            y = self.hl[i].sub(&x);

            x_right.push(x);
            y_right.push(y);
        }

        return (x_left, x_right, y_left, y_right);
    }
}

pub fn vec_to_u32(digits: &Vec<char>) -> Option<u32> {
    const RADIX: u32 = 10;
    return digits
        .iter()
        .map(|c| c.to_digit(RADIX))
        .try_fold(0, |ans, i| i.map(|i| ans * RADIX + i));
}
