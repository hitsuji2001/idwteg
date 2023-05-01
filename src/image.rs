use std::collections::VecDeque;
use std::{fs, io::Write, str};

use crate::color::RGBColor;

#[derive(Debug)]
pub struct PPMImage {
    pub img_type: String,
    pub width: usize,
    pub height: usize,
    pub max_val: usize,
    pub data: Vec<RGBColor<i32>>,
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

pub fn vec_to_u32(digits: &Vec<char>) -> Option<u32> {
    const RADIX: u32 = 10;
    return digits
        .iter()
        .map(|c| c.to_digit(RADIX))
        .try_fold(0, |ans, i| i.map(|i| ans * RADIX + i));
}
