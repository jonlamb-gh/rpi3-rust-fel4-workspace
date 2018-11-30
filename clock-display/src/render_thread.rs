use bcm2837_hal::bcm2837::dma::{DMA, ENABLE};
use bcm2837_hal::dma::DmaExt;
use bcm2837_hal::mailbox_msg::PixelOrder;
use bcm2837_hal::pmem::PMem;
use display::{Display, ObjectDrawing};
use embedded_graphics::coord::Coord;
use rgb::RGB8;
use sel4_sys::seL4_Word;

use clock::{Clock, Config as ClockConfig};

pub const FAULT_EP_BADGE: seL4_Word = 0xDEAD;
pub const IPC_EP_BADGE: seL4_Word = 0xBEEF;

// 8 x 4K pages
pub const STACK_SIZE_PAGES: usize = 8;

#[repr(C)]
#[derive(Debug)]
pub struct Config {
    pub dma_vaddr: seL4_Word,
    pub scratchpad_pmem: PMem,
    pub fb_width: usize,
    pub fb_height: usize,
    pub fb_pitch: usize,
    pub fb_pixel_order: PixelOrder,
    pub framebuffer_pmem: PMem,
    pub backbuffer_pmem: PMem,
}

// TODO
// impl Config fn is_valid()

pub fn run(config_vaddr: seL4_Word) {
    debug_println!("\nRender thread running\n");

    // TODO - sanity check fields
    assert_ne!(config_vaddr, 0);
    let config = unsafe { &*(config_vaddr as *const Config) };
    //debug_println!("{:#?}", config);

    // Construct the DMA peripheral, reset and enable CH0
    let dma = DMA::from(config.dma_vaddr);
    let dma_parts = dma.split();
    dma_parts.enable.ENABLE.write(ENABLE::EN0::SET);
    dma_parts.ch0.reset();

    let mut display = Display::new(
        dma_parts.ch0,
        config.fb_width,
        config.fb_height,
        config.fb_pitch,
        config.fb_pixel_order,
        config.scratchpad_pmem,
        config.framebuffer_pmem,
        config.backbuffer_pmem,
    );

    let mut clock = Clock::new(ClockConfig {
        center: Coord::new(display.width() as i32 / 2, display.height() as i32 / 2),
        radius: (display.height() as u32 / 2) - 1,
        outline_stroke_width: 4,
        outline_color: RGB8::new(0xFF, 0xFF, 0xFF),
    });

    let mut hour: u32 = 0;
    let mut min: u32 = 0;
    let mut sec: u32 = 0;

    // Clear back and front buffers
    display.clear_screen();

    loop {
        // Clear the backbuffer
        display.clear_buffer();

        // Move the clock digits around
        sec += 1;
        if sec >= 60 {
            sec = 0;
            min += 1;
            if min >= 60 {
                min = 0;
                hour += 1;
                if hour >= 12 {
                    hour = 0;
                }
            }
        }

        clock.update_digits(hour, min, sec);

        clock.draw_object(&mut display);

        display.swap_buffers();
    }
}
