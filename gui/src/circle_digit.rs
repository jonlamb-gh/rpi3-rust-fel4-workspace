// TODO
// - error/sanity checks
// - use Style<RGB8>?
// - iterator

use core::fmt::Write;
use display::{Display, DisplayColor, ObjectDrawing};
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::{Font, Font12x16};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Circle;
use heapless::consts::U32;
use heapless::String;
use rgb::RGB8;

// TODO - use Style<RGB8>?
#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub center: Coord,
    pub radius: u32,
    pub fill: bool,
    pub text_color: RGB8,
    pub background_fill_color: RGB8,
    pub stroke_color: RGB8,
    pub stroke_width: u8,
}

pub struct CircleDigit {
    config: Config,
    value_str: String<U32>,
    value: u32,
}

impl CircleDigit {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            value_str: String::from("0"),
            value: 0,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn set_value(&mut self, value: u32) {
        self.value = value;
        self.value_str.clear();
        write!(self.value_str, "{}", self.value).ok();
    }

    pub fn set_center(&mut self, coord: Coord) {
        self.config.center = coord;
    }

    fn draw_circle(&self, display: &mut Display) {
        let mut circle: Circle<DisplayColor> = Circle::new(self.config.center, self.config.radius)
            .with_stroke(Some(self.config.stroke_color.into()))
            .with_stroke_width(self.config.stroke_width);

        if self.config.fill {
            circle = circle.with_fill(Some(self.config.background_fill_color.into()));
        }

        display.draw(circle.into_iter());
    }

    fn draw_text(&self, display: &mut Display) {
        let text: Font12x16<DisplayColor> =
            Font12x16::render_str(&self.value_str).with_stroke(Some(self.config.text_color.into()));

        let text_coord = Coord::new(
            self.config.center.0 - (text.dimensions().0 as i32 / 2),
            self.config.center.1 - (text.dimensions().1 as i32 / 2),
        );

        display.draw(
            text.with_fill(Some(self.config.background_fill_color.into()))
                .translate(text_coord)
                .into_iter(),
        );
    }
}

impl ObjectDrawing for CircleDigit {
    fn draw_object(&self, display: &mut Display) {
        self.draw_circle(display);
        self.draw_text(display);
    }
}
