// TODO
// - error/sanity checks
// - horizontal/vertical
// - use Style<RGB8>?
// - iterator
// - impl Drawable for BarGraph {}

use core::fmt::Write;
use display::{Display, DisplayColor, ObjectDrawing};
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font;
use embedded_graphics::fonts::Font12x16;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rect;
use heapless::consts::U32;
use heapless::String;
use rgb::RGB8;

// TODO - use Style<RGB8>?
#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub top_left: Coord,
    pub bottom_right: Coord,
    pub background_color: RGB8,
    pub fill_color: RGB8,
    pub text_color: RGB8,
    pub stroke_color: RGB8, //style: Style
    pub stroke_width: u8,
}

// Storing extra state to keep the drawing loops as slim as possible
pub struct BarGraph {
    config: Config,
    value_str: String<U32>,
    value: f32,
    height: i32,
    center_x: i32,
    fill_dist: i32,
}

const TEXT_VERT_PADDING: i32 = 3;

impl BarGraph {
    pub fn new(config: Config) -> Self {
        // precompute some commonly used bits
        let width: i32 = config.bottom_right.abs().0 - config.top_left.abs().0;
        let height: i32 = config.bottom_right.abs().1 - config.top_left.abs().1;
        let center_x: i32 = config.top_left.0 + (width / 2);

        Self {
            config,
            value_str: String::new(),
            value: 0.0,
            height,
            center_x,
            fill_dist: 0,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = if value <= 0.0 {
            0.0
        } else if value >= 1.0 {
            1.0
        } else {
            value
        };

        let scaled = self.value * (self.height as f32);
        self.fill_dist = scaled as i32;

        write!(self.value_str, "{:.*}", 0, 100.0 * self.value).ok();
    }

    fn draw_fillings(&self, display: &mut Display) {
        if self.fill_dist <= 0 {
            // empty
            display.draw(
                Rect::new(self.config.top_left, self.config.bottom_right)
                    .with_fill(Some(self.config.background_color.into()))
                    .into_iter(),
            );
        } else if self.fill_dist >= self.height {
            // full
            display.draw(
                Rect::new(self.config.top_left, self.config.bottom_right)
                    .with_fill(Some(self.config.fill_color.into()))
                    .into_iter(),
            );
        } else {
            // in between, start with the background color
            display.draw(
                Rect::new(
                    self.config.top_left,
                    Coord::new(
                        self.config.bottom_right.0,
                        self.config.bottom_right.1 - self.fill_dist,
                    ),
                )
                .with_fill(Some(self.config.background_color.into()))
                .into_iter(),
            );

            // graph fill color
            display.draw(
                Rect::new(
                    Coord::new(
                        self.config.top_left.0,
                        self.config.bottom_right.1 - self.fill_dist,
                    ),
                    self.config.bottom_right,
                )
                .with_fill(Some(self.config.fill_color.into()))
                .into_iter(),
            );
        }
    }

    fn draw_value_text(&self, display: &mut Display) {
        let text: Font12x16<DisplayColor> =
            Font12x16::render_str(&self.value_str).with_stroke(Some(self.config.text_color.into()));

        let room_needed = self.height - (text.dimensions().1 as i32) - (4 * TEXT_VERT_PADDING);
        let room_above = if self.fill_dist <= room_needed {
            true
        } else {
            false
        };

        // put the text above the fill line if we have room
        let (text_coord, text_bg_color) = if room_above {
            (
                Coord::new(
                    self.center_x - (text.dimensions().0 as i32 / 2),
                    self.config.bottom_right.1
                        - self.fill_dist
                        - (text.dimensions().1 as i32)
                        - TEXT_VERT_PADDING,
                ),
                self.config.background_color,
            )
        } else {
            // otherwise put it below
            (
                Coord::new(
                    self.center_x - (text.dimensions().0 as i32 / 2),
                    self.config.bottom_right.1 - self.fill_dist + TEXT_VERT_PADDING,
                ),
                self.config.fill_color,
            )
        };

        display.draw(
            text.with_fill(Some(text_bg_color.into()))
                .translate(text_coord)
                .into_iter(),
        );
    }

    fn draw_outline_rect(&self, display: &mut Display) {
        display.draw(
            Rect::new(self.config.top_left, self.config.bottom_right)
                .with_stroke(Some(self.config.stroke_color.into()))
                .with_stroke_width(self.config.stroke_width)
                .into_iter(),
        );
    }
}

impl ObjectDrawing for BarGraph {
    fn draw_object(&self, display: &mut Display) {
        self.draw_fillings(display);
        self.draw_value_text(display);
        self.draw_outline_rect(display);
    }
}
