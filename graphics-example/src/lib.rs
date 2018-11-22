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
use gui::*;
use rgb::RGB8;
use sel4_sys::*;
use sel4twinkle_alloc::{Allocator, DMACacheOp, PMem, PAGE_BITS_4K, PAGE_SIZE_4K};

#[macro_use]
mod macros;

pub fn handle_fault(badge: seL4_Word) {
    debug_println!("\n!!! Fault from badge 0x{:X}\n", badge);
}

pub fn init(allocator: &mut Allocator, _global_fault_ep_cap: seL4_CPtr) {
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
    let desired_width = 800;
    let desired_height = 480;
    let fb_cfg = FramebufferCmd {
        phy_width: desired_width,
        phy_height: desired_height,
        virt_width: desired_width,
        virt_height: desired_height,
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

    let mut display = Display::new(
        fb_resp.phy_width,
        fb_resp.phy_height,
        fb_resp.pitch,
        pmem.vaddr,
    );

    //display.fill_color(0xFF00FF_u32.into());

    let bar_graph_config = BarGraphConfig {
        top_left: Coord::new(100, 50),
        bottom_right: Coord::new(150, 250),
        background_color: RGB8::new(0xF0, 0x0F, 0xCF),
        fill_color: RGB8::new(0x00, 0xAF, 0xCF),
        text_color: RGB8::new(0xFF, 0xFF, 0xFF),
        stroke_color: RGB8::new(0xFF, 0xFF, 0xFF),
        stroke_width: 2,
    };

    let mut bar_graph = BarGraph::new(bar_graph_config);

    let mut circle_digit = CircleDigit::new(CircleDigitConfig {
        center: Coord::new(300, 200),
        radius: 30,
        fill: true,
        text_color: RGB8::new(0xFF, 0xFF, 0xFF),
        background_fill_color: RGB8::new(0xAF, 0xAF, 0x00),
        stroke_color: RGB8::new(0xFF, 0xFF, 0xFF),
        stroke_width: 2,
    });

    bar_graph.set_value(0.90);
    bar_graph.draw_object(&mut display);

    circle_digit.set_value(4);
    circle_digit.draw_object(&mut display);
}
