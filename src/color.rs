#[derive(Debug, Copy, Clone)]
pub struct RGBColor<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
}

impl<T: Copy> RGBColor<T> {
    pub fn new(red: T, green: T, blue: T) -> RGBColor<T> {
        return RGBColor { red, green, blue };
    }

    pub fn add(self, color: &RGBColor<T>) -> RGBColor<T>
    where
        T: std::ops::Add<Output = T>,
    {
        return RGBColor {
            red: self.red + color.red,
            green: self.green + color.green,
            blue: self.blue + color.blue,
        };
    }

    pub fn sub(self, color: &RGBColor<T>) -> RGBColor<T>
    where
        T: std::ops::Sub<Output = T>,
    {
        return RGBColor {
            red: self.red - color.red,
            green: self.green - color.green,
            blue: self.blue - color.blue,
        };
    }

    pub fn div_by(self, number: T) -> RGBColor<T>
    where
        T: std::ops::Div<Output = T>,
    {
        return RGBColor {
            red: self.red / number,
            green: self.green / number,
            blue: self.blue / number,
        };
    }
}
