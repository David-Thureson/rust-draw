pub type Color = [f32; 4];

#[derive(Clone, Debug)]
pub struct Color1 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Clone, Debug)]
pub struct Color256 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color1 {
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        debug_assert!(r >= 0.0);
        debug_assert!(r <= 1.0);
        debug_assert!(g >= 0.0);
        debug_assert!(g <= 1.0);
        debug_assert!(b >= 0.0);
        debug_assert!(b <= 1.0);
        debug_assert!(a >= 0.0);
        debug_assert!(a <= 1.0);
        Self {
            r,
            g,
            b,
            a
        }
    }

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::from_rgba(r, g, b, 1.0)
    }

    pub fn black() -> Self {
        Self::from_rgb(0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::from_rgb(1.0, 1.0, 1.0)
    }

    pub fn red() -> Self {
        Self::from_rgb(1.0, 0.0, 0.0)
    }

    pub fn green() -> Self {
        Self::from_rgb(0.0, 1.0, 0.0)
    }

    pub fn blue() -> Self {
        Self::from_rgb(0.0, 0.0, 1.0)
    }
}

impl From<Color256> for Color1 {
    fn from(color: Color256) -> Self {
        let (r, g, b, a) = color.into();
        Self::from_rgba(r as f32 / 256.0, g as f32 / 256.0, b as f32 / 256.0, a as f32 / 256.0)
    }
}

impl Into<[f32; 4]> for Color1 {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Color256 {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

}

impl Into<(u8, u8, u8, u8)> for Color256 {
    fn into(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}