use embedded_graphics::pixelcolor::PixelColor;
use rgb::{alt::BGR8, RGB8};

/// A wrapper around RGB8 for now
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DisplayColor(pub RGB8);

impl PixelColor for DisplayColor {}

impl From<u8> for DisplayColor {
    #[inline]
    fn from(other: u8) -> Self {
        DisplayColor(RGB8::new(other, other, other))
    }
}

impl From<u16> for DisplayColor {
    #[inline]
    fn from(other: u16) -> Self {
        let mono = (other >> 1 & 0xFF) as u8;
        DisplayColor(RGB8::new(mono, mono, mono))
    }
}

impl From<u32> for DisplayColor {
    #[inline]
    fn from(other: u32) -> Self {
        DisplayColor(RGB8::new(
            (other & 0xFF) as u8,
            (other >> 8 & 0xFF) as u8,
            (other >> 16 & 0xFF) as u8,
        ))
    }
}

impl From<(u8, u8, u8)> for DisplayColor {
    #[inline]
    fn from(other: (u8, u8, u8)) -> Self {
        DisplayColor(RGB8::new(other.0, other.1, other.2))
    }
}

impl From<RGB8> for DisplayColor {
    #[inline]
    fn from(other: RGB8) -> Self {
        DisplayColor(other)
    }
}

impl From<DisplayColor> for u32 {
    #[inline]
    fn from(color: DisplayColor) -> u32 {
        0xFF_00_00_00 | color.0.r as u32 | (color.0.g as u32) << 8 | (color.0.b as u32) << 16
    }
}

impl DisplayColor {
    pub fn into_inner(self) -> RGB8 {
        self.0
    }

    pub fn into_inner_alt(self) -> BGR8 {
        BGR8::from(self.0)
    }

    // TODO - this is not idiomatic...
    pub fn as_alt(&self) -> u32 {
        0xFF_00_00_00 | self.0.b as u32 | (self.0.g as u32) << 8 | (self.0.r as u32) << 16
    }
}
