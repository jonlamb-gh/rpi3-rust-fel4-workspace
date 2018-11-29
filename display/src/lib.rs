#![no_std]

// TODO
// - fix the broken DMA logic when src == scratchpad_buffer, dst == backbuffer?
// - use embedded-graphics types/traits on Display (top-left()/etc)
// - configs for single/double buffer modes
// - better management/book-keeping of physical/virtual addresses
// - better DisplayColor and pixel order handling, move to display_color.rs?

extern crate bcm2837_hal;
extern crate embedded_graphics;
extern crate rgb;

mod display_color;

use bcm2837_hal::dma;
use bcm2837_hal::mailbox_msg::PixelOrder;
use core::ptr;
use embedded_graphics::drawable::Pixel;
use embedded_graphics::Drawing;

pub use display_color::DisplayColor;

// TODO - until I figure out how to cleanly use embedded-graphics IntoIterator
// to combine primitives,
// this can be used to pass around a mut Display
pub trait ObjectDrawing {
    fn draw_object(&self, display: &mut Display);
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
    scratchpad_vaddr: u64,
    scratchpad_paddr: u32,
    width: u32,
    height: u32,
    pitch: u32,
    pixel_order: PixelOrder,
    fb_paddr: u32,
    fb_ptr: *mut u32,
    fb_backbuffer_size: usize,
    fb_backbuffer_paddr: u32,
    fb_backbuffer_ptr: *mut u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TransferOp {
    /// Fill the backbuffer with a value
    FillBack,
    /// Fill the frontbuffer with a value
    FillFront,
    /// Copy the back buffer to the frontbuffer (typically GPU memory)
    CopyBackToFront,
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
        fb_backbuffer_vaddr: u64,
        fb_backbuffer_paddr: u32,
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

        let control_blocks = unsafe {
            core::slice::from_raw_parts_mut(
                (scratchpad_vaddr + SP_CONTROL_BLOCK_OFFSET as u64) as *mut dma::ControlBlock,
                NUM_CONTROL_BLOCKS as _,
            )
        };

        for cb in control_blocks.iter_mut() {
            cb.init();
        }

        Self {
            dma,
            scratchpad_vaddr,
            scratchpad_paddr,
            width,
            height,
            pitch,
            pixel_order,
            fb_paddr,
            fb_ptr: fb_vaddr as *mut u32,
            fb_backbuffer_size: (width * height * 4) as _,
            fb_backbuffer_paddr,
            fb_backbuffer_ptr: fb_backbuffer_vaddr as *mut u32,
        }
    }

    /// Sets a pixel in the backbuffer
    /// RGB b[0] = Red, b[1] = Green, b[2] = Blue, b[3] = NA
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u32) {
        let color_word: u32 = if self.pixel_order == PixelOrder::RGB {
            value
        } else {
            DisplayColor::from(value).as_alt()
        };

        // The frontbuffer, may not be contiguous so must use pitch (pitch >= bpp*width)
        //let offset = (y * (self.pitch / 4)) + x;
        //unsafe { ptr::write(self.fb_ptr.offset(offset as _), color_word) };

        // The backbuffer is contiguous
        let offset = (y * self.width) + x;
        unsafe { ptr::write(self.fb_backbuffer_ptr.offset(offset as _), color_word) };
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Clears the backbuffer and the frontbuffer
    pub fn clear_screen(&mut self) {
        self.set_scratchpad_src_fill_words(0_u32.into());

        //self.dma_transfer(TransferOp::FillFront);

        // TODO - DMA to backbuffer currently broken
        //self.dma_transfer(TransferOp::FillBack);
        // Clear it manually for now
        self.fill_pixels(0_u32.into());
        self.swap_buffers();
    }

    /// Clears the backbuffer
    pub fn clear_buffer(&mut self) {
        // TODO - public buffer enum type?
        self.set_scratchpad_src_fill_words(0_u32.into());

        // TODO - DMA to backbuffer currently broken
        //self.dma_transfer(TransferOp::FillBack);
        // Clear it manually for now
        self.fill_pixels(0_u32.into());
    }

    pub fn swap_buffers(&mut self) {
        self.dma_transfer(TransferOp::CopyBackToFront);
    }

    /// Fills the backbuffer with a color using a DMA transfer
    pub fn fill_color(&mut self, color: DisplayColor) {
        self.set_scratchpad_src_fill_words(color);

        // TODO - DMA to backbuffer currently broken
        //self.dma_transfer(TransferOp::FillBack);
        // Fill it manually for now
        self.fill_pixels(color);
    }

    /// Fills the backbuffer with a color pixel by pixel
    pub fn fill_pixels(&mut self, color: DisplayColor) {
        /*
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color.into());
            }
        }
        */

        // Since the backbuffer is contiguous, we can use memset/alike
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(self.fb_backbuffer_ptr, self.fb_backbuffer_size / 4)
        };
        let color_word: u32 = if self.pixel_order == PixelOrder::RGB {
            color.into()
        } else {
            color.as_alt()
        };

        for word in buffer.iter_mut() {
            *word = color_word;
        }
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

    fn dma_transfer(&mut self, op: TransferOp) {
        // Stride, in bytes, is a signed inc/dec applied after end of each row
        let bbp: u32 = 4;
        let frontbuffer_stride = self.pitch - (self.width * bbp);
        let backbuffer_stride = 0;

        // Both the backbuffer and the scratchpad words are contiguous
        let src_stride = 0;

        let (src_inc, src_paddr, dst_paddr, dst_stride) = match op {
            TransferOp::FillBack => {
                // Filling the backbuffer with the contents of the scratchpad words
                (
                    false,
                    self.scratchpad_paddr + SP_FILL_WORDS_OFFSET,
                    self.fb_backbuffer_paddr,
                    backbuffer_stride,
                )
            }
            TransferOp::FillFront => {
                // Filling the frontbuffer with the contents of the scratchpad words
                (
                    false,
                    self.scratchpad_paddr + SP_FILL_WORDS_OFFSET,
                    self.fb_paddr,
                    frontbuffer_stride,
                )
            }
            TransferOp::CopyBackToFront => {
                // Copy the backbuffer to the frontbuffer
                (
                    true,
                    self.fb_backbuffer_paddr,
                    self.fb_paddr,
                    frontbuffer_stride,
                )
            }
        };

        // This is not really obvious from the DMA documentation,
        // but the top 16 bits must be programmmed to "height -1"
        // and not "height" in 2D mode.
        let transfer_length = dma::TransferLength::Mode2D(
            // transfer length in bytes of a row
            (bbp * self.width) as _,
            // How many x-length transfers are performed
            (self.height - 1) as _,
        );

        let cb_config = dma::ControlBlockConfig {
            int_enable: false,
            transfer_length,
            wait_for_resp: true,
            dest_inc: true,
            dest_width_128: true,
            dest_dreq: false,
            dest_ignore: false,
            src_inc,
            src_width_128: true,
            src_dreq: false,
            src_ignore: false,
            burst_length: 4,
            peripheral_map: 0,
            waits: 0,
            no_wide_bursts: false,
        };

        let control_blocks = unsafe {
            core::slice::from_raw_parts_mut(
                (self.scratchpad_vaddr + SP_CONTROL_BLOCK_OFFSET as u64) as *mut dma::ControlBlock,
                NUM_CONTROL_BLOCKS as _,
            )
        };

        // Apply control block configuration to the control block
        control_blocks[0].config(
            &cb_config,
            src_paddr,
            dst_paddr,
            src_stride as _,
            dst_stride as _,
            0,
        );

        // Wait for DMA to be ready, then do the transfer
        while self.dma.is_busy() == true {}
        self.dma
            .start(self.scratchpad_paddr + SP_CONTROL_BLOCK_OFFSET);
        self.dma.wait();

        assert_eq!(self.dma.errors(), false, "DMA errors present");
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
