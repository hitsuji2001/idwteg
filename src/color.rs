#[derive(Debug, Copy, Clone)]
pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct RGBColorFlexer {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
}

impl RGBColor {
    pub fn new(red: u8, green: u8, blue: u8) -> RGBColor {
        return RGBColor { red, green, blue };
    }

    pub fn add(self, color: &RGBColor) -> RGBColorFlexer {
        return RGBColorFlexer {
            red: self.red as i32 + color.red as i32,
            green: self.green as i32 + color.green as i32,
            blue: self.blue as i32 + color.blue as i32,
        };
    }

    pub fn subtract(self, color: &RGBColor) -> RGBColorFlexer {
        return RGBColorFlexer {
            red: self.red as i32 - color.red as i32,
            green: self.green as i32 - color.green as i32,
            blue: self.blue as i32 - color.blue as i32,
        };
    }
}

impl RGBColorFlexer {
    pub fn add(self, color: &RGBColorFlexer) -> RGBColorFlexer {
        return RGBColorFlexer {
            red: self.red + color.red,
            green: self.green + color.green,
            blue: self.blue + color.blue,
        };
    }

    pub fn subtract(self, color: &RGBColorFlexer) -> RGBColorFlexer {
        return RGBColorFlexer {
            red: self.red - color.red,
            green: self.green - color.green,
            blue: self.blue - color.blue,
        };
    }
}
