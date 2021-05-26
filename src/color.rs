#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [
            self.0[0] as f32 / 255.,
            self.0[1] as f32 / 255.,
            self.0[2] as f32 / 255.,
            self.0[3] as f32 / 255.,
        ]
    }
}

impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        self.0
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [
            self.0[0] as f32 / 255.,
            self.0[1] as f32 / 255.,
            self.0[2] as f32 / 255.,
        ]
    }
}

impl From<[u8; 3]> for Color {
    fn from(rgb: [u8; 3]) -> Self {
        Color([rgb[0], rgb[1], rgb[2], 255u8])
    }
}

impl From<[f32; 3]> for Color {
    fn from(rgb: [f32; 3]) -> Self {
        Color::new(rgb[0], rgb[1], rgb[2], 1f32)
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color([
            (r.min(1.).max(0.) * 255.) as u8,
            (g.min(1.).max(0.) * 255.) as u8,
            (b.min(1.).max(0.) * 255.) as u8,
            (a.min(1.).max(0.) * 255.) as u8,
        ])
    }
}

#[allow(dead_code)]
pub mod colors {
    use super::Color;
    pub const LIGHTGRAY: Color = Color([200, 200, 200, 255]);
    pub const GRAY: Color = Color([130, 130, 130, 255]);
    pub const DARKGRAY: Color = Color([80, 80, 80, 255]);
    pub const YELLOW: Color = Color([253, 249, 0, 255]);
    pub const GOLD: Color = Color([255, 203, 0, 255]);
    pub const ORANGE: Color = Color([255, 161, 0, 255]);
    pub const PINK: Color = Color([255, 109, 194, 255]);
    pub const RED: Color = Color([230, 41, 55, 255]);
    pub const MAROON: Color = Color([190, 33, 55, 255]);
    pub const GREEN: Color = Color([0, 228, 48, 255]);
    pub const LIME: Color = Color([0, 158, 47, 255]);
    pub const DARKGREEN: Color = Color([0, 117, 44, 255]);
    pub const SKYBLUE: Color = Color([102, 191, 255, 255]);
    pub const BLUE: Color = Color([0, 121, 241, 255]);
    pub const DARKBLUE: Color = Color([0, 82, 172, 255]);
    pub const PURPLE: Color = Color([200, 122, 255, 255]);
    pub const VIOLET: Color = Color([135, 60, 190, 255]);
    pub const DARKPURPLE: Color = Color([112, 31, 126, 255]);
    pub const BEIGE: Color = Color([211, 176, 131, 255]);
    pub const BROWN: Color = Color([127, 106, 79, 255]);
    pub const DARKBROWN: Color = Color([76, 63, 47, 255]);
    pub const WHITE: Color = Color([255, 255, 255, 255]);
    pub const BLACK: Color = Color([0, 0, 0, 255]);
    pub const BLANK: Color = Color([0, 0, 0, 0]);
    pub const MAGENTA: Color = Color([255, 0, 255, 255]);
}
