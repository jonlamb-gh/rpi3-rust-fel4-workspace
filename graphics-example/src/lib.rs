#![no_std]

extern crate bcm2837_hal;
extern crate display;
extern crate embedded_graphics;
extern crate gui;
extern crate rgb;
extern crate sel4_sys;
extern crate sel4twinkle_alloc;

use bcm2837_hal::bcm2837::dma::ENABLE;
use bcm2837_hal::bcm2837::dma::{DMA, PADDR as DMA_PADDR};
use bcm2837_hal::bcm2837::mbox::{
    BASE_OFFSET as MBOX_BASE_OFFSET, BASE_PADDR as MBOX_BASE_PADDR, MBOX,
};
use bcm2837_hal::dma;
use bcm2837_hal::dma::DmaExt;
use bcm2837_hal::mailbox::{Channel, Mailbox};
use bcm2837_hal::mailbox_msg::*;
use display::Display;
use display::ObjectDrawing;
use embedded_graphics::coord::Coord;
use gui::*;
use rgb::RGB8;
use sel4_sys::*;
use sel4twinkle_alloc::{Allocator, DMACacheOp, InitCap, PMem, PAGE_BITS_4K, PAGE_SIZE_4K};

const DISPLAY_WIDTH: u32 = 600;
const DISPLAY_HEIGHT: u32 = 300;

const FAULT_EP_BADGE: seL4_Word = 0xDEAD;
const IPC_EP_BADGE: seL4_Word = 0xBEEF;

// 8 x 4K pages
const THREAD_STACK_NUM_PAGES: usize = 8;

#[macro_use]
mod macros;

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

    debug_println!("Allocated mbox buffer pmem page");
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

    let dma_vaddr = allocator
        .io_map(DMA_PADDR, PAGE_BITS_4K as _)
        .expect("Failed to io_map");

    debug_println!("Mapped DMA device region");
    debug_println!("  vaddr = 0x{:X} paddr = 0x{:X}", dma_vaddr, DMA_PADDR);

    // Allocate a page of memory to hold the DMA control blocks
    let num_cb = PAGE_SIZE_4K as usize / dma::CONTROL_BLOCK_SIZE as usize;
    let dma_cb_pmem = allocator
        .pmem_new_dma_page(None)
        .expect("Failed to allocate pmem");

    debug_println!(
        "Allocated DMA control block(s) pmem page, holds {} control blocks",
        num_cb,
    );
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X}",
        dma_cb_pmem.vaddr,
        dma_cb_pmem.paddr,
    );

    allocator.dma_cache_op(
        dma_cb_pmem.vaddr,
        PAGE_SIZE_4K as _,
        DMACacheOp::CleanInvalidate,
    );

    debug_println!("\nRequesting framebuffer\n");

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
    let gpu_pmem = allocator
        .pmem_new_pages_at_paddr(
            fb_resp.paddr as _,
            pages as _,
            // Not cacheable
            0,
        ).expect("pmem_new_pages_at_paddr");

    allocator.dma_cache_op(
        gpu_pmem.vaddr,
        mem_size_bytes as _,
        DMACacheOp::CleanInvalidate,
    );

    // Create an IPC buffer / page of memory to store the thread data parameters
    let thread_data_vaddr = allocator
        .vspace_new_ipc_buffer(None)
        .expect("Failed to allocate thread data memory");

    let thread_data = unsafe { &mut *(thread_data_vaddr as *mut ThreadData) };
    thread_data.dma_vaddr = dma_vaddr;
    thread_data.scratchpad_vaddr = dma_cb_pmem.vaddr;
    thread_data.scratchpad_paddr = dma_cb_pmem.paddr;
    thread_data.fb_width = fb_resp.phy_width;
    thread_data.fb_height = fb_resp.phy_height;
    thread_data.fb_pitch = fb_resp.pitch;
    thread_data.fb_pixel_order = fb_resp.pixel_order;
    thread_data.fb_vaddr = gpu_pmem.vaddr;
    thread_data.fb_paddr = fb_resp.bus_paddr.into();
    // TODO
    //thread_data.fb_paddr = fb_resp.paddr.into();

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
            Some(thread_data_vaddr),
            None,
            None,
        ).expect("Failed to configure thread");

    thread
        .start(InitCap::InitThreadTCB.into())
        .expect("Failed to start thread");
}

#[repr(C)]
#[derive(Debug)]
struct ThreadData {
    dma_vaddr: seL4_Word,
    scratchpad_vaddr: seL4_Word,
    scratchpad_paddr: seL4_Word,
    fb_width: u32,
    fb_height: u32,
    fb_pitch: u32,
    fb_pixel_order: PixelOrder,
    fb_vaddr: seL4_Word,
    fb_paddr: seL4_Word,
}

fn render_thread_function(thread_data_vaddr: seL4_Word) {
    debug_println!("\nRender thread running\n");
    assert_ne!(thread_data_vaddr, 0);

    // TODO - sanity check fields
    let thread_data = unsafe { &*(thread_data_vaddr as *const ThreadData) };

    debug_println!("{:#?}", thread_data);

    let dma = DMA::from(thread_data.dma_vaddr);

    // Split into the various channels/etc
    let dma_parts = dma.split();

    dma_parts.enable.ENABLE.write(ENABLE::EN0::SET);

    dma_parts.ch0.reset();

    let mut display = Display::new(
        dma_parts.ch0,
        thread_data.scratchpad_vaddr,
        thread_data.scratchpad_paddr as _,
        thread_data.fb_width,
        thread_data.fb_height,
        thread_data.fb_pitch,
        thread_data.fb_pixel_order,
        thread_data.fb_vaddr,
        thread_data.fb_paddr as _,
    );

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

    let mut float_val: f32 = 0.0;
    let mut u_val: u32 = 0;

    loop {
        display.clear_screen();

        bar_graph.set_value(float_val);
        bar_graph.draw_object(&mut display);

        circle_digit.set_value(u_val);
        circle_digit.draw_object(&mut display);

        float_val += 0.1;
        if float_val > 1.0 {
            float_val = 0.0;
        }

        u_val += 1;
    }
}
