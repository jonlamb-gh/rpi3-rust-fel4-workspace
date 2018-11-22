#![no_std]

extern crate bcm2837_hal;
extern crate display;
extern crate embedded_graphics;
extern crate gui;
extern crate rgb;
extern crate sel4_sys;
extern crate sel4twinkle_alloc;

use bcm2837_hal::bcm2837::mbox::{
    BASE_OFFSET as MBOX_BASE_OFFSET, BASE_PADDR as MBOX_BASE_PADDR, MBOX,
};
use bcm2837_hal::mailbox::{Channel, Mailbox};
use bcm2837_hal::mailbox_msg::*;
use display::Display;
use display::ObjectDrawing;
use embedded_graphics::coord::Coord;
use rgb::RGB8;
use sel4_sys::*;
use sel4twinkle_alloc::{Allocator, DMACacheOp, InitCap, PMem, PAGE_BITS_4K, PAGE_SIZE_4K};

#[macro_use]
mod macros;
mod clock;

use clock::{Clock, Config as ClockConfig};

const DISPLAY_WIDTH: u32 = 800;
const DISPLAY_HEIGHT: u32 = 480;

const FAULT_EP_BADGE: seL4_Word = 0xDEAD;
const IPC_EP_BADGE: seL4_Word = 0xBEEF;

// 4 x 4K pages
const THREAD_STACK_NUM_PAGES: usize = 4;

pub fn handle_fault(badge: seL4_Word) {
    debug_println!("\n!!! Fault from badge 0x{:X}\n", badge);
}

pub fn init(allocator: &mut Allocator, global_fault_ep_cap: seL4_CPtr) {
    // VideoCore Mailbox
    let base_size = PAGE_BITS_4K as usize;
    let vc_mbox_paddr: seL4_Word = MBOX_BASE_PADDR + MBOX_BASE_OFFSET;

    let base_vaddr = allocator
        .io_map(MBOX_BASE_PADDR, base_size)
        .expect("Failed to io_map");

    let vc_mbox_vaddr = base_vaddr + MBOX_BASE_OFFSET;

    debug_println!("Mapped VideoCore Mailbox device region");
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X}",
        vc_mbox_vaddr,
        vc_mbox_paddr,
    );
    debug_println!(
        "  base vaddr = 0x{:X} base_paddr 0x{:X}",
        base_vaddr,
        MBOX_BASE_PADDR,
    );

    // Allocate a new page of memory with a physical address
    // so we can give it to the VideoCore
    let mbox_buffer_pmem: PMem = allocator
        .pmem_new_dma_page(None)
        .expect("Failed to allocate pmem/DMA page");

    allocator.dma_cache_op(
        mbox_buffer_pmem.vaddr,
        PAGE_SIZE_4K as _,
        DMACacheOp::CleanInvalidate,
    );

    debug_println!("Allocated pmem page");
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X}",
        mbox_buffer_pmem.vaddr,
        mbox_buffer_pmem.paddr
    );

    let mut mbox: Mailbox = Mailbox::new(
        MBOX::from(vc_mbox_vaddr),
        mbox_buffer_pmem.paddr as _,
        mbox_buffer_pmem.vaddr as _,
    );

    debug_println!("\nCreating a Display\n");

    // TODO - need to go enable full GPU region in the kernel devices
    let fb_cfg = FramebufferCmd {
        phy_width: DISPLAY_WIDTH,
        phy_height: DISPLAY_HEIGHT,
        virt_width: DISPLAY_WIDTH,
        virt_height: DISPLAY_HEIGHT,
        x_offset: 0,
        y_offset: 0,
    };

    let resp: Resp = mbox
        .call(Channel::Prop, &fb_cfg)
        .expect("Mailbox::call failed");

    let fb_resp = if let Resp::FramebufferResp(r) = resp {
        r
    } else {
        panic!("Bad response {:#?}", resp);
    };

    debug_println!("{:#?}", fb_resp);

    assert_eq!(fb_resp.phy_width, DISPLAY_WIDTH);
    assert_eq!(fb_resp.phy_height, DISPLAY_HEIGHT);

    // TODO - should be virt, pitch * virt_height
    let mem_size_bytes = (fb_resp.phy_height * fb_resp.pitch) as seL4_Word;
    let pages = 1 + mem_size_bytes / PAGE_SIZE_4K;

    // Map in the GPU memory
    let pmem = allocator
        .pmem_new_pages_at_paddr(
            fb_resp.paddr as _,
            pages as _,
            // Not cacheable
            0,
        ).expect("pmem_new_pages_at_paddr");

    allocator.dma_cache_op(pmem.vaddr, mem_size_bytes as _, DMACacheOp::CleanInvalidate);

    let mut thread = allocator
        .create_thread(
            global_fault_ep_cap,
            FAULT_EP_BADGE,
            IPC_EP_BADGE,
            THREAD_STACK_NUM_PAGES,
        ).expect("Failed to create thread");

    thread
        .configure_context(
            render_thread_function as _,
            Some(pmem.vaddr),
            Some(fb_resp.pitch as seL4_Word),
            None,
        ).expect("Failed to configure thread");

    thread
        .start(InitCap::InitThreadTCB.into())
        .expect("Failed to start thread");
}

fn render_thread_function(fb_vaddr: seL4_Word, fb_pitch: seL4_Word) {
    debug_println!("\nRender thread running\n");
    assert_ne!(fb_vaddr, 0);
    assert_ne!(fb_pitch, 0);

    let mut display = Display::new(DISPLAY_WIDTH, DISPLAY_HEIGHT, fb_pitch as _, fb_vaddr as _);

    let mut clock = Clock::new(ClockConfig {
        center: Coord::new(display.width() as i32 / 2, display.height() as i32 / 2),
        radius: (display.height() / 2) - 1,
        outline_stroke_width: 4,
        outline_color: RGB8::new(0xFF, 0xFF, 0xFF),
    });

    clock.draw_object(&mut display);

    let mut hour: u32 = 0;
    let mut min: u32 = 0;
    let mut sec: u32 = 0;

    loop {
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

        display.fill_color(0_u32.into());

        clock.draw_object(&mut display);
    }
}
