#![no_std]

// TODO
// - fix the broken DMA logic when src == scratchpad_buffer, dst == backbuffer?
// - use embedded-graphics types/traits on Display (top-left()/etc)
// - configs for single/double buffer modes

extern crate bcm2837_hal;
extern crate embedded_graphics;
extern crate rgb;

mod display_color;

use bcm2837_hal::dma;
use bcm2837_hal::mailbox_msg::PixelOrder;
use bcm2837_hal::pmem::PMem;
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
const SP_CONTROL_BLOCK_OFFSET: usize = 0;

/// Offset into the scratchpad buffer used to store
/// a fill word in DMA fill operations.
/// The first word in the last CONTROL_BLOCK_SIZE bytes
const SP_FILL_WORDS_OFFSET: usize = NUM_CONTROL_BLOCKS * dma::CONTROL_BLOCK_SIZE;

const PAGE_SIZE_4K: usize = 1 << 12;
const NUM_CONTROL_BLOCKS: usize = (PAGE_SIZE_4K / dma::CONTROL_BLOCK_SIZE) - 1;

/// Fill words to be used by the DMA engine when doing color fills, up to
/// 128 bit writes are supported
const NUM_FILL_WORDS: usize = 4;

#[derive(Debug)]
pub struct Display {
    dma: dma::Channel,
    width: u32,
    height: u32,
    pitch: u32,
    pixel_order: PixelOrder,
    scratchpad: PMem,
    /// Control blocks and fill words are split pmem from the provided
    /// scratchpad
    control_blocks: PMem,
    fill_words: PMem,
    /// Framebuffer is also the front buffer
    framebuffer: PMem,
    backbuffer: PMem,
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
        width: u32,
        height: u32,
        pitch: u32,
        pixel_order: PixelOrder,
        scratchpad: PMem,
        framebuffer: PMem,
        backbuffer: PMem,
    ) -> Self {
        assert_eq!(
            dma.is_lite(),
            false,
            "Can't use a lite DMA engine for 2D transfers"
        );
        assert_ne!(width, 0);
        assert_ne!(height, 0);
        assert_ne!(pitch, 0);

        let control_blocks_pmem = PMem::new(
            scratchpad.vaddr() + SP_CONTROL_BLOCK_OFFSET as u64,
            scratchpad.paddr() + SP_CONTROL_BLOCK_OFFSET as u32,
            NUM_CONTROL_BLOCKS * dma::CONTROL_BLOCK_SIZE,
        );

        for cb in control_blocks_pmem
            .as_mut_slice::<dma::ControlBlock>(NUM_CONTROL_BLOCKS)
            .iter_mut()
        {
            cb.init();
        }

        let fill_words_pmem = PMem::new(
            scratchpad.vaddr() + SP_FILL_WORDS_OFFSET as u64,
            scratchpad.paddr() + SP_FILL_WORDS_OFFSET as u32,
            NUM_FILL_WORDS * 4,
        );

        Self {
            dma,
            width,
            height,
            pitch,
            pixel_order,
            scratchpad,
            control_blocks: control_blocks_pmem,
            fill_words: fill_words_pmem,
            framebuffer,
            backbuffer,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
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
        //unsafe { ptr::write(self.framebuffer.as_mut_ptr()
        //.offset(offset as _), color_word) };

        // The backbuffer is contiguous
        let offset = (y * self.width) + x;
        unsafe {
            ptr::write(
                self.backbuffer.as_mut_ptr::<u32>().offset(offset as _),
                color_word,
            )
        };
    }

    /// Clears the backbuffer and the frontbuffer
    pub fn clear_screen(&mut self) {
        self.set_scratchpad_src_fill_words(0_u32.into());

        // self.dma_transfer(TransferOp::FillFront);

        // TODO - DMA to backbuffer currently broken
        // self.dma_transfer(TransferOp::FillBack);
        // Clear it manually for now
        self.fill_pixels(0_u32.into());
        self.swap_buffers();
    }

    /// Clears the backbuffer
    pub fn clear_buffer(&mut self) {
        // TODO - public buffer enum type?
        self.set_scratchpad_src_fill_words(0_u32.into());

        // TODO - DMA to backbuffer currently broken
        // self.dma_transfer(TransferOp::FillBack);
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
        // self.dma_transfer(TransferOp::FillBack);
        // Fill it manually for now
        self.fill_pixels(color);
    }

    /// Fills the backbuffer with a color pixel by pixel
    pub fn fill_pixels(&mut self, color: DisplayColor) {
        // Since the backbuffer is contiguous, we can use memset/alike
        let buffer = self
            .backbuffer
            .as_mut_slice::<u32>(self.backbuffer.size() / 4);
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
        let fill_words = self.fill_words.as_mut_slice::<u32>(NUM_FILL_WORDS);

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
                    self.fill_words.paddr(),
                    self.backbuffer.paddr(),
                    backbuffer_stride,
                )
            }
            TransferOp::FillFront => {
                // Filling the frontbuffer with the contents of the scratchpad words
                (
                    false,
                    self.fill_words.paddr(),
                    self.framebuffer.paddr(),
                    frontbuffer_stride,
                )
            }
            TransferOp::CopyBackToFront => {
                // Copy the backbuffer to the frontbuffer
                (
                    true,
                    self.backbuffer.paddr(),
                    self.framebuffer.paddr(),
                    frontbuffer_stride,
                )
            }
        };

        // This is not really obvious from the DMA documentation,
        // but the top 16 bits must be programmmed to "height -1"
        // and not "height" in 2D mode.
        let transfer_length = dma::TransferLength::Mode2D(
            // Transfer length in bytes of a row
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

        let control_blocks = self
            .control_blocks
            .as_mut_slice::<dma::ControlBlock>(NUM_CONTROL_BLOCKS);

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
        self.dma.start(self.control_blocks.paddr());
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
