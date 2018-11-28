#![no_std]

// TODO
// - use embedded-graphics types/traits on Display
// - local render buffer for double buffering, swap does a DMA transfer
// - handle PixelOrder

extern crate bcm2837_hal;
extern crate embedded_graphics;
extern crate rgb;

use bcm2837_hal::dma;
use bcm2837_hal::mailbox_msg::PixelOrder;
use core::ptr;
use embedded_graphics::drawable::Pixel;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::Drawing;
use rgb::*;

// TODO - until I figure out how to cleanly use embedded-graphics IntoIterator
// to combine primitives,
// this can be used to pass around a mut Display
pub trait ObjectDrawing {
    fn draw_object(&self, display: &mut Display);
}

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
}

/// Offset into the scratchpad buffer used to store
/// the DMA control block
const SP_CONTROL_BLOCK_OFFSET: u32 = 0;

/// Offset into the scratchpad buffer used to store
/// a fill word in DMA fill operations.
/// The first word in the last CONTROL_BLOCK_SIZE bytes
const SP_FILL_WORDS_OFFSET: u32 = NUM_CONTROL_BLOCKS * dma::CONTROL_BLOCK_SIZE as u32;

const PAGE_SIZE_4K: u32 = 1 << 12;
const NUM_CONTROL_BLOCKS: u32 = (PAGE_SIZE_4K / dma::CONTROL_BLOCK_SIZE) - 1;

const NUM_FILL_WORDS: u32 = 4;

#[derive(Debug)]
pub struct Display {
    dma: dma::Channel,
    scratchpad_paddr: u32,
    scratchpad_vaddr: u64,
    width: u32,
    height: u32,
    pitch: u32,
    pixel_order: PixelOrder,
    fb_paddr: u32,
    fb_ptr: *mut u32,
}

impl Display {
    /// Expects to be given at least 1 4K page of DMA scratchpad mem
    pub fn new(
        dma: dma::Channel,
        scratchpad_vaddr: u64,
        scratchpad_paddr: u32,
        width: u32,
        height: u32,
        pitch: u32,
        pixel_order: PixelOrder,
        fb_vaddr: u64,
        fb_paddr: u32,
    ) -> Self {
        assert_eq!(
            dma.is_lite(),
            false,
            "Can't use a lite DMA engine for 2D transfers"
        );
        assert_ne!(scratchpad_vaddr, 0);
        assert_ne!(scratchpad_paddr, 0);
        assert_ne!(width, 0);
        assert_ne!(height, 0);
        assert_ne!(pitch, 0);
        assert_ne!(fb_paddr, 0);
        assert_ne!(fb_vaddr, 0);

        Self {
            dma,
            scratchpad_paddr,
            scratchpad_vaddr,
            width,
            height,
            pitch,
            pixel_order,
            fb_paddr,
            fb_ptr: fb_vaddr as *mut u32,
        }
    }

    /// RGB b[0] = Red, b[1] = Green, b[2] = Blue, b[3] = NA
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u32) {
        let offset = (y * (self.pitch / 4)) + x;
        unsafe { ptr::write(self.fb_ptr.offset(offset as _), value) };
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /*
    pub fn fill_color(&mut self, color: DisplayColor) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color.into());
            }
        }
    }
    */

    pub fn clear_screen(&mut self) {
        self.fill_color(0_u32.into());
    }

    pub fn fill_color(&mut self, color: DisplayColor) {
        // Put the color in the fill word
        self.set_scratchpad_src_fill_words(color);

        // Construct a control block config for the DMA transfer
        let mut cb_config = dma::ControlBlockConfig::default();
        cb_config.dest_inc = true;
        cb_config.dest_width_128 = true;
        cb_config.src_width_128 = true;
        cb_config.src_inc = false;
        cb_config.burst_length = 4;
        cb_config.wait_for_resp = true;

        // Stride, in bytes, is a signed inc/dec applied after end of each row
        let bbp: u32 = 4;
        let stride = self.pitch - (self.width * bbp);

        // This is not really obvious from the DMA documentation,
        // but the top 16 bits must be programmmed to "height -1"
        // and not "height" in 2D mode.
        cb_config.transfer_length = dma::TransferLength::Mode2D(
            // transfer length in bytes of a row
            (bbp * self.width) as _,
            // How many x-length transfers are performed
            (self.height - 1) as _,
        );

        let control_blocks = unsafe {
            core::slice::from_raw_parts_mut(
                (self.scratchpad_vaddr + SP_CONTROL_BLOCK_OFFSET as u64) as *mut dma::ControlBlock,
                NUM_CONTROL_BLOCKS as _,
            )
        };

        // Apply control block configuration to the control block
        control_blocks[0].init();
        control_blocks[0].config(
            &cb_config,
            self.scratchpad_paddr + SP_FILL_WORDS_OFFSET,
            self.fb_paddr,
            stride as _,
            stride as _,
            0,
        );

        // Wait for DMA to be ready, then do the transfer
        while self.dma.is_busy() == true {}
        self.dma
            .start(self.scratchpad_paddr + SP_CONTROL_BLOCK_OFFSET);
        self.dma.wait();

        assert_eq!(self.dma.errors(), false, "DMA errors present");
    }

    /// Constructs the DMA source fill words in the internal scratchpad buffer
    fn set_scratchpad_src_fill_words(&mut self, color: DisplayColor) {
        let fill_words = unsafe {
            core::slice::from_raw_parts_mut(
                (self.scratchpad_vaddr + SP_FILL_WORDS_OFFSET as u64) as *mut u32,
                NUM_FILL_WORDS as _,
            )
        };

        for w in fill_words.iter_mut() {
            *w = color.into();
        }
    }
}

impl Drawing<DisplayColor> for Display {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<DisplayColor>>,
    {
        for Pixel(coord, color) in item_pixels {
            if coord[0] >= self.width || coord[1] >= self.height {
                continue;
            }

            self.set_pixel(coord[0], coord[1], u32::from(color));
        }
    }
}
