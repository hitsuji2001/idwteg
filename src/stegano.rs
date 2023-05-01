use std::cmp::Ordering;

use crate::color::RGBColor;
use crate::image::PPMImage;

type Block<T> = [RGBColor<T>; 4];
const IH_INDEX: usize = 0;
const IV_INDEX: usize = 1;
const ID_INDEX: usize = 2;

#[derive(Debug)]
pub struct DWTImage {
    pub ll: Vec<RGBColor<i32>>, // approximation coefficients
    pub lh: Vec<RGBColor<i32>>, // vertical details
    pub hl: Vec<RGBColor<i32>>, // horizontal details
    pub hh: Vec<RGBColor<i32>>, // diagonal details
    pub orig_width: usize,
    pub orig_height: usize,
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

    pub fn hide_image(
        orig_img_file_path: &str,
        secret_img_file_path: &str,
    ) -> std::io::Result<(Vec<usize>, Vec<(usize, usize)>, usize, usize)> {
        let orginal_image = DWTImage::from_ppm(&PPMImage::from_file(&orig_img_file_path));
        let message_image = DWTImage::from_ppm(&PPMImage::from_file(&secret_img_file_path));
        let (watermarked_image, key, index_arr) = orginal_image.hide_message(&message_image);
        watermarked_image.export_to_file("./images/watermarked.ppm")?;

        Ok((
            key,
            index_arr,
            message_image.orig_width,
            message_image.orig_height,
        ))
    }

    pub fn extract_message_from_image(
        file_path: &str,
        output_file_path: &str,
        key1: Vec<usize>,
        key2: Vec<(usize, usize)>,
        orig_width: usize,
        orig_height: usize,
    ) -> std::io::Result<()> {
        let ppm_img = PPMImage::from_file(&file_path);
        let secret = DWTImage::from_ppm(&ppm_img); // DWT transform
        let (ia, ih, iv, id) = (
            DWTImage::blocking_extract_one(&secret.ll, secret.orig_width, secret.orig_height),
            DWTImage::blocking_extract_one(&secret.lh, secret.orig_width, secret.orig_height),
            DWTImage::blocking_extract_one(&secret.hl, secret.orig_width, secret.orig_height),
            DWTImage::blocking_extract_one(&secret.hh, secret.orig_width, secret.orig_height),
        ); // Blocking
        let mut bia = Vec::<Block<i32>>::new();
        for i in 0..key1.len() {
            bia.push(ia[key1[i]]);
        }
        let (mut dih, mut div, mut did) = (
            Vec::<Block<i32>>::new(),
            Vec::<Block<i32>>::new(),
            Vec::<Block<i32>>::new(),
        );

        let max_width = (orig_width * orig_height) / 4;
        for _ in 0..max_width {
            dih.push([RGBColor::<i32>::default(); 4]);
            div.push([RGBColor::<i32>::default(); 4]);
            did.push([RGBColor::<i32>::default(); 4]);
        }

        for i in 0..key2.len() {
            if key2[i].0 == IH_INDEX {
                dih[key2[i].1] = DWTImage::block_add(&bia[i], &ih[key2[i].1]);
            } else if key2[i].0 == IV_INDEX {
                div[key2[i].1] = DWTImage::block_add(&bia[i], &iv[key2[i].1]);
            } else if key2[i].0 == ID_INDEX {
                did[key2[i].1] = DWTImage::block_add(&bia[i], &id[key2[i].1]);
            }
        }

        let image = DWTImage::rearrange_blocks(&ia, &dih, &div, &did, orig_width, orig_height);
        image.inverse_dwt().export_to_file(&output_file_path)?;
        Ok(())
    }

    fn hide_message(&self, mess: &DWTImage) -> (PPMImage, Vec<usize>, Vec<(usize, usize)>) {
        // blocking
        let (ia, mut ih, mut iv, mut id, sa) = (
            DWTImage::blocking_extract_one(&self.ll, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&self.lh, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&self.hl, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&self.hh, self.orig_width, self.orig_height),
            DWTImage::blocking_extract_one(&mess.ll, mess.orig_width, mess.orig_height),
        );
        let key1 = DWTImage::matching(&sa, &ia);
        let mut bd = DWTImage::block_differences_computation(&sa, &ia, &key1);
        let index_arr = DWTImage::block_replacement(&mut bd, &mut ih, &mut iv, &mut id);
        let watermarked_image =
            DWTImage::rearrange_blocks(&ia, &ih, &iv, &id, self.orig_width, self.orig_height);

        return (watermarked_image.inverse_dwt(), key1, index_arr);
    }

    fn inverse_dwt(&self) -> PPMImage {
        let mut result_image = PPMImage::new();
        let (x_left, x_right, y_left, y_right) = self.inverse_vertical_transform();
        let secret_image = DWTImage::inverse_horizontal_transform(x_left, x_right, y_left, y_right);
        let mut block_count = 0;

        result_image.img_type = String::from("P6");
        result_image.width = self.orig_width;
        result_image.height = self.orig_height;
        result_image.max_val = 255;
        result_image.data = vec![RGBColor::new(0, 0, 0); self.orig_width * self.orig_height];

        // Learn how to cast struct pls
        for y in (0..self.orig_height).step_by(2) {
            for x in (0..self.orig_width).step_by(2) {
                result_image.data[(y + 0) * self.orig_width + (x + 0)] = RGBColor::new(
                    secret_image[block_count][0].red as i32,
                    secret_image[block_count][0].green as i32,
                    secret_image[block_count][0].blue as i32,
                );
                // secret_image[block_count][0];
                result_image.data[(y + 0) * self.orig_width + (x + 1)] = RGBColor::new(
                    secret_image[block_count][1].red as i32,
                    secret_image[block_count][1].green as i32,
                    secret_image[block_count][1].blue as i32,
                );
                // secret_image[block_count][1];
                result_image.data[(y + 1) * self.orig_width + (x + 0)] = RGBColor::new(
                    secret_image[block_count][2].red as i32,
                    secret_image[block_count][2].green as i32,
                    secret_image[block_count][2].blue as i32,
                );
                // secret_image[block_count][2];
                result_image.data[(y + 1) * self.orig_width + (x + 1)] = RGBColor::new(
                    secret_image[block_count][3].red as i32,
                    secret_image[block_count][3].green as i32,
                    secret_image[block_count][3].blue as i32,
                );
                // secret_image[block_count][3];
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
            DWTImage::rearrange_one_block(&ia, width / 2, height / 2),
            DWTImage::rearrange_one_block(&ih, width / 2, height / 2),
            DWTImage::rearrange_one_block(&iv, width / 2, height / 2),
            DWTImage::rearrange_one_block(&id, width / 2, height / 2),
        );

        return DWTImage::new(ll, lh, hl, hh, width, height);
    }

    fn rearrange_one_block(
        arr: &Vec<Block<i32>>,
        width: usize,
        height: usize,
    ) -> Vec<RGBColor<i32>> {
        let mut result: Vec<RGBColor<i32>> = vec![RGBColor::new(0, 0, 0); width * height];
        let mut block_count = 0;

        for y in (0..height - 1).step_by(2) {
            for x in (0..width - 1).step_by(2) {
                result[(y + 0) * width + (x + 0)] = arr[block_count][0];
                result[(y + 0) * width + (x + 1)] = arr[block_count][1];
                result[(y + 1) * width + (x + 0)] = arr[block_count][2];
                result[(y + 1) * width + (x + 1)] = arr[block_count][3];
                block_count += 1;
            }
        }

        return result;
    }

    fn block_replacement(
        bd: &mut Vec<Block<i32>>,
        ih: &mut Vec<Block<i32>>,
        iv: &mut Vec<Block<i32>>,
        id: &mut Vec<Block<i32>>,
    ) -> Vec<(usize, usize)> {
        let mut index_arr = Vec::<(usize, usize)>::new();
        for i in 0..bd.len() {
            let (ih_index, iv_index, id_index) = (
                DWTImage::find_most_fit_block_index(&bd[i], &ih),
                DWTImage::find_most_fit_block_index(&bd[i], &iv),
                DWTImage::find_most_fit_block_index(&bd[i], &id),
            );
            let elements = vec![ih[ih_index], iv[iv_index], id[id_index]];

            let mut result = Vec::<(f64, usize)>::new();
            result.push((
                DWTImage::root_mean_square_error(&bd[i], &elements[0]),
                ih_index,
            ));
            result.push((
                DWTImage::root_mean_square_error(&bd[i], &elements[1]),
                iv_index,
            ));
            result.push((
                DWTImage::root_mean_square_error(&bd[i], &elements[2]),
                id_index,
            ));

            quicksort::quicksort_by(&mut result, DWTImage::float_usize_tuple_compare);
            match result[0].1 {
                x if x == ih_index => {
                    ih[ih_index] = bd[i];
                    index_arr.push((IH_INDEX, ih_index));
                }
                x if x == iv_index => {
                    iv[iv_index] = bd[i];
                    index_arr.push((IV_INDEX, iv_index));
                }
                x if x == id_index => {
                    id[id_index] = bd[i];
                    index_arr.push((ID_INDEX, id_index));
                }
                _ => {
                    eprintln!("Unreachable, index = {}", result[0].1);
                    panic!();
                }
            }
        }
        return index_arr;
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
        let mut result = Vec::<(f64, usize)>::new();
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
        index_arr: &Vec<usize>,
    ) -> Vec<Block<i32>> {
        let mut result = Vec::<Block<i32>>::new();
        for sa_index in 0..sa.len() {
            result.push(DWTImage::block_sub(&sa[sa_index], &ia[index_arr[sa_index]]));
        }
        return result;
    }

    fn root_mean_square_error(vec1: &Block<i32>, vec2: &Block<i32>) -> f64 {
        let mut result_color = RGBColor::<f64>::new(0.0, 0.0, 0.0);

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
        let mut result = Vec::<Block<i32>>::new();
        let mut temp_arr: Block<i32> = [RGBColor::new(0, 0, 0); 4];

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

    fn matching(sa: &Vec<Block<i32>>, ia: &Vec<Block<i32>>) -> Vec<usize> {
        let mut result: Vec<Vec<(f64, usize)>> = vec![Vec::new(); sa.len()];
        let mut index_arr = Vec::<usize>::new();
        for ia_index in 0..ia.len() {
            for sa_index in 0..sa.len() {
                result[sa_index].push((
                    DWTImage::root_mean_square_error(&sa[sa_index], &ia[ia_index]),
                    ia_index,
                ));
            }
        }

        for i in 0..sa.len() {
            quicksort::quicksort_by(&mut result[i], DWTImage::float_usize_tuple_compare);
            index_arr.push(result[i][0].1);
        }

        return index_arr;
    }

    fn inverse_horizontal_transform(
        x_left: Vec<RGBColor<f64>>,
        x_right: Vec<RGBColor<f64>>,
        y_left: Vec<RGBColor<f64>>,
        y_right: Vec<RGBColor<f64>>,
    ) -> Vec<Block<f64>> {
        let mut result = Vec::<Block<f64>>::new();
        let mut x: RGBColor<f64>;
        let mut y: RGBColor<f64>;
        let mut temp_block: Block<f64> = [RGBColor::new(0., 0., 0.); 4];

        for i in 0..x_left.len() {
            x = (x_left[i].add(&x_right[i])).div_by(2.0);
            y = x_left[i].sub(&x);
            temp_block[0] = x;
            temp_block[1] = y;
            x = (y_left[i].add(&y_right[i])).div_by(2.0);
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
        Vec<RGBColor<f64>>,
        Vec<RGBColor<f64>>,
        Vec<RGBColor<f64>>,
        Vec<RGBColor<f64>>,
    ) {
        let mut x_left = Vec::<RGBColor<f64>>::new();
        let mut x_right = Vec::<RGBColor<f64>>::new();
        let mut y_left = Vec::<RGBColor<f64>>::new();
        let mut y_right = Vec::<RGBColor<f64>>::new();
        let mut x: RGBColor<f64>;
        let mut y: RGBColor<f64>;

        // FIXME: Learn how to cast from generic type to primitive type pls
        let (self_ll, self_lh, self_hl, self_hh) = self.convert_to_f64();

        for i in 0..self.ll.len() {
            x = (self_ll[i].add(&self_lh[i])).div_by(2.0);
            y = self_ll[i].sub(&x);

            x_left.push(x);
            y_left.push(y);

            x = (self_hl[i].add(&self_hh[i])).div_by(2.0);
            y = self_hl[i].sub(&x);

            x_right.push(x);
            y_right.push(y);
        }

        return (x_left, x_right, y_left, y_right);
    }

    // This function shouldn't exist just like this entire code base
    fn convert_to_f64(
        &self,
    ) -> (
        Vec<RGBColor<f64>>,
        Vec<RGBColor<f64>>,
        Vec<RGBColor<f64>>,
        Vec<RGBColor<f64>>,
    ) {
        let (mut self_ll, mut self_lh, mut self_hl, mut self_hh) = (
            Vec::<RGBColor<f64>>::new(),
            Vec::<RGBColor<f64>>::new(),
            Vec::<RGBColor<f64>>::new(),
            Vec::<RGBColor<f64>>::new(),
        );

        for i in 0..self.ll.len() {
            self_ll.push(RGBColor::new(
                self.ll[i].red as f64,
                self.ll[i].green as f64,
                self.ll[i].blue as f64,
            ));
            self_lh.push(RGBColor::new(
                self.lh[i].red as f64,
                self.lh[i].green as f64,
                self.lh[i].blue as f64,
            ));
            self_hl.push(RGBColor::new(
                self.hl[i].red as f64,
                self.hl[i].green as f64,
                self.hl[i].blue as f64,
            ));
            self_hh.push(RGBColor::new(
                self.hh[i].red as f64,
                self.hh[i].green as f64,
                self.hh[i].blue as f64,
            ));
        }

        return (self_ll, self_lh, self_hl, self_hh);
    }

    fn block_sub<T>(b1: &Block<T>, b2: &Block<T>) -> Block<T>
    where
        T: std::ops::Sub<Output = T> + Copy + Default,
    {
        let mut result: Block<T> = [RGBColor::<T>::default(); 4];
        for i in 0..4 {
            result[i] = b1[i].sub(&b2[i]);
        }

        return result;
    }

    fn block_add<T>(b1: &Block<T>, b2: &Block<T>) -> Block<T>
    where
        T: std::ops::Add<Output = T> + Copy + Default,
    {
        let mut result: Block<T> = [RGBColor::<T>::default(); 4];
        for i in 0..4 {
            result[i] = b1[i].add(&b2[i]);
        }

        return result;
    }
}
