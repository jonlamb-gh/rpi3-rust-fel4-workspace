#![no_std]

extern crate bcm2837_hal;
extern crate display;
extern crate embedded_graphics;
extern crate gui;
extern crate rgb;
extern crate sel4_sys;
extern crate sel4twinkle_alloc;

use bcm2837_hal::bcm2837::dma::PADDR as DMA_PADDR;
use bcm2837_hal::bcm2837::mbox::{
    BASE_OFFSET as MBOX_BASE_OFFSET, BASE_PADDR as MBOX_BASE_PADDR, MBOX,
};
use bcm2837_hal::mailbox::{Channel, Mailbox};
use bcm2837_hal::mailbox_msg::*;
use bcm2837_hal::pmem::PMem as HALPMem;
use sel4_sys::*;
use sel4twinkle_alloc::{Allocator, DMACacheOp, InitCap, PAGE_BITS_4K, PAGE_SIZE_4K};

#[macro_use]
mod macros;
mod clock;
mod render_thread;

const DISPLAY_WIDTH: usize = 800;
const DISPLAY_HEIGHT: usize = 480;

pub fn handle_fault(badge: seL4_Word) {
    debug_println!("\n!!! Fault from badge 0x{:X}\n", badge);
}

pub fn init(allocator: &mut Allocator, global_fault_ep_cap: seL4_CPtr) {
    debug_println!("Mapping VideoCore mailbox device");
    let vc_mbox_dev_pmem = map_device_pmem(
        allocator,
        MBOX_BASE_PADDR,
        PAGE_BITS_4K as _,
        MBOX_BASE_OFFSET as _,
    );

    debug_println!("Mapping DMA device");
    let dma_dev_pmem = map_device_pmem(allocator, DMA_PADDR, PAGE_BITS_4K as _, 0);

    let display_backbuffer_size = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;

    // Size in bytes of the dma pool to reserve
    let dma_pool_size: seL4_Word =
        // 1 page for the mailbox buffer
        PAGE_SIZE_4K +
        // 1 page for the display scratchpad buffer, holds DMA control blocks/etc
        PAGE_SIZE_4K +
        // Pages for the contiguous display backbuffer, always 4 bpp
        display_backbuffer_size as seL4_Word;
    let dma_pool_size_pages = 1 + (dma_pool_size / PAGE_SIZE_4K);

    debug_println!(
        "Reserving DMA pool size 0x{:X} ({}), {} pages",
        dma_pool_size,
        dma_pool_size,
        dma_pool_size_pages
    );

    // Allocate/map the dma pool, more effecient to use the pool currently
    // since my allocator doesn't book-keep untypes/retypes
    // NOTE: this only works if there is some untyped large enough, might have to
    // split it up
    let mut dma_pool_pmem = reserve_dma_pool(allocator, dma_pool_size_pages as _);

    // Split off a page from the DMA pool for the mailbox buffer
    let mbox_buffer_pmem = dma_pool_pmem.split(PAGE_SIZE_4K as _);

    allocator.dma_cache_op(
        mbox_buffer_pmem.vaddr(),
        mbox_buffer_pmem.size(),
        DMACacheOp::CleanInvalidate,
    );

    debug_println!("Mailbox buffer DMA pmem");
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X} size = 0x{:X}",
        mbox_buffer_pmem.vaddr(),
        mbox_buffer_pmem.paddr(),
        mbox_buffer_pmem.size(),
    );

    // Split off a page from the DMA pool for the display scratchpad buffer
    let display_scratchpad_pmem = dma_pool_pmem.split(PAGE_SIZE_4K as _);

    allocator.dma_cache_op(
        display_scratchpad_pmem.vaddr(),
        display_scratchpad_pmem.size(),
        DMACacheOp::CleanInvalidate,
    );

    debug_println!("Display scratchpad DMA pmem");
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X} size = 0x{:X}",
        display_scratchpad_pmem.vaddr(),
        display_scratchpad_pmem.paddr(),
        display_scratchpad_pmem.size(),
    );

    // Use the remaining memory in the DMA pool as the display backbuffer
    dma_pool_pmem.reduce_to(display_backbuffer_size);
    let display_backbuffer_pmem = dma_pool_pmem;

    allocator.dma_cache_op(
        display_backbuffer_pmem.vaddr(),
        display_backbuffer_pmem.size(),
        DMACacheOp::CleanInvalidate,
    );

    debug_println!("Display backbuffer DMA pmem");
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X} size = 0x{:X}",
        display_backbuffer_pmem.vaddr(),
        display_backbuffer_pmem.paddr(),
        display_backbuffer_pmem.size(),
    );

    let mut mbox: Mailbox = Mailbox::new(MBOX::from(vc_mbox_dev_pmem.vaddr()), mbox_buffer_pmem);

    let mut framebuffer_pitch: usize = 0;
    let mut framebuffer_pixel_order: PixelOrder = PixelOrder::RGB;
    let display_framebuffer_pmem = request_framebuffer(
        allocator,
        &mut mbox,
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        &mut framebuffer_pitch,
        &mut framebuffer_pixel_order,
    );

    debug_println!("Mapped in VideoCore GPU pmem");
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X} size = 0x{:X}",
        display_framebuffer_pmem.vaddr(),
        display_framebuffer_pmem.paddr(),
        display_framebuffer_pmem.size(),
    );

    // Create an IPC buffer / page of memory to store the thread data parameters
    let thread_data_vaddr = allocator
        .vspace_new_ipc_buffer(None)
        .expect("Failed to allocate thread data memory");

    // Fill the thread config parameters
    let thread_data = unsafe { &mut *(thread_data_vaddr as *mut render_thread::Config) };
    thread_data.dma_vaddr = dma_dev_pmem.vaddr();
    thread_data.scratchpad_pmem = display_scratchpad_pmem;
    thread_data.fb_width = DISPLAY_WIDTH;
    thread_data.fb_height = DISPLAY_HEIGHT;
    thread_data.fb_pitch = framebuffer_pitch;
    thread_data.fb_pixel_order = framebuffer_pixel_order;
    thread_data.framebuffer_pmem = display_framebuffer_pmem;
    thread_data.backbuffer_pmem = display_backbuffer_pmem;

    let mut thread = allocator
        .create_thread(
            global_fault_ep_cap,
            render_thread::FAULT_EP_BADGE,
            render_thread::IPC_EP_BADGE,
            render_thread::STACK_SIZE_PAGES,
        ).expect("Failed to create thread");

    thread
        .configure_context(render_thread::run as _, Some(thread_data_vaddr), None, None)
        .expect("Failed to configure thread");

    thread
        .start(InitCap::InitThreadTCB.into())
        .expect("Failed to start thread");
}

fn request_framebuffer(
    allocator: &mut Allocator,
    mbox: &mut Mailbox,
    width: usize,
    height: usize,
    framebuffer_pitch: &mut usize,
    framebuffer_pixel_order: &mut PixelOrder,
) -> HALPMem {
    let fb_cfg = FramebufferCmd {
        phy_width: width as _,
        phy_height: height as _,
        virt_width: width as _,
        virt_height: height as _,
        x_offset: 0,
        y_offset: 0,
    };

    let resp: Resp = mbox
        .call(Channel::Prop, &fb_cfg)
        .expect("Mailbox::call failed on FramebufferCmd");

    let fb_resp = if let Resp::FramebufferResp(r) = resp {
        r
    } else {
        panic!("Mailbox returned an invalid response {:#?}", resp);
    };

    assert_eq!(fb_resp.phy_width as usize, width);
    assert_eq!(fb_resp.phy_height as usize, height);

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
        ).expect("Failed to map in GPU framebuffer memory");

    allocator.dma_cache_op(
        gpu_pmem.vaddr,
        mem_size_bytes as _,
        DMACacheOp::CleanInvalidate,
    );

    *framebuffer_pitch = fb_resp.pitch as _;
    *framebuffer_pixel_order = fb_resp.pixel_order;

    HALPMem::new(
        gpu_pmem.vaddr,
        // Use the bus address as given by the VideoCore
        fb_resp.bus_paddr,
        mem_size_bytes as _,
    )
}

// TODO - result/error-handling
fn map_device_pmem(
    allocator: &mut Allocator,
    paddr: seL4_Word,
    size_bits: usize,
    offset: usize,
) -> HALPMem {
    let size = 1 << size_bits;
    assert!(
        offset < size,
        "Offset {} must be less than size {}",
        offset,
        size
    );

    let base_paddr = paddr;
    let base_vaddr = allocator
        .io_map(base_paddr, size_bits)
        .expect("Failed to io_map");

    debug_println!(
        "  dev_pmem base_vaddr = 0x{:X} base_paddr = 0x{:X} size = 0x{:X} offset = 0x{:X}",
        base_vaddr,
        base_paddr,
        size,
        offset
    );

    HALPMem::new(
        (base_vaddr + offset as seL4_Word) as _,
        (base_paddr + offset as seL4_Word) as _,
        size - offset,
    )
}

// TODO - result/error-handling
fn reserve_dma_pool(allocator: &mut Allocator, num_pages: usize) -> HALPMem {
    let pmem = allocator
        .pmem_new_dma_pages(
            num_pages as _,
            // Not cacheable
            0,
        ).expect("Failed to reserve DMA pool");

    let size = num_pages * PAGE_SIZE_4K as usize;

    debug_println!(
        "  dma_pmem vaddr = 0x{:X} paddr = 0x{:X} size = 0x{:X}",
        pmem.vaddr,
        pmem.paddr,
        size
    );

    HALPMem::new(pmem.vaddr, pmem.paddr as _, size)
}
