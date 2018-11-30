#![no_std]

extern crate bcm2837_hal;
extern crate sel4_sys;
extern crate sel4twinkle_alloc;

use bcm2837_hal::bcm2837::dma::ENABLE;
use bcm2837_hal::bcm2837::dma::{DMA, PADDR as DMA_PADDR};
use bcm2837_hal::bcm2837::mbox::{
    BASE_OFFSET as MBOX_BASE_OFFSET, BASE_PADDR as MBOX_BASE_PADDR, MBOX,
};
use bcm2837_hal::dma::*;
use bcm2837_hal::mailbox::{Channel, Mailbox};
use bcm2837_hal::mailbox_msg::*;
use bcm2837_hal::pmem::PMem as HALPMem;
use core::ptr;
use sel4_sys::*;
use sel4twinkle_alloc::{Allocator, DMACacheOp, PMem, PAGE_BITS_4K, PAGE_SIZE_4K};

#[macro_use]
mod macros;

pub fn handle_fault(badge: seL4_Word) {
    debug_println!("\n!!! Fault from badge 0x{:X}\n", badge);
}

pub fn init(allocator: &mut Allocator, _global_fault_ep_cap: seL4_CPtr) {
    debug_println!("\nHello from custom init fn\n");

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
    // TODO
    //let mbox_buffer_pmem: PMem = allocator.pmem_new_page(None)
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

    // Mailbox
    let mut mbox: Mailbox = Mailbox::new(
        MBOX::from(vc_mbox_vaddr),
        HALPMem::new(
            mbox_buffer_pmem.vaddr,
            mbox_buffer_pmem.paddr as _,
            PAGE_SIZE_4K as _,
        ),
    );

    // DMA
    let dma_vaddr = allocator
        .io_map(DMA_PADDR, PAGE_BITS_4K as _)
        .expect("Failed to io_map");

    debug_println!("Mapped DMA device region");
    debug_println!("  vaddr = 0x{:X} paddr = 0x{:X}", dma_vaddr, DMA_PADDR);

    let dma = DMA::from(dma_vaddr);

    // Split into the various channels/etc
    let dma_parts = dma.split();

    debug_println!("DMA Parts = \n{:#?}", dma_parts);

    debug_println!(
        "DMA channel 0 - ID: {} - is_lite: {}",
        dma_parts.ch0.dma_id(),
        dma_parts.ch0.is_lite(),
    );

    debug_println!(
        "DMA channel 1 - ID: {} - is_lite: {}",
        dma_parts.ch1.dma_id(),
        dma_parts.ch1.is_lite(),
    );

    // TODO - pmem page, slice from, slice of control blocks

    // Allocate a page of memory to hold the DMA control blocks
    // Reserves the last CONTROL_BLOCK_SIZE bytes for some scratchpad bytes
    let num_cb = (PAGE_SIZE_4K as usize / CONTROL_BLOCK_SIZE as usize) - 1;
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

    // Scratchpad bytes at the end of the page
    let scratchpad_offset = num_cb as u32 * CONTROL_BLOCK_SIZE as u32;
    let scratchpad_vaddr = dma_cb_pmem.vaddr as u32 + scratchpad_offset;
    let scratchpad_paddr = dma_cb_pmem.paddr as u32 + scratchpad_offset;
    let scratchpad_ptr = scratchpad_vaddr as *mut u32;

    // We fill the screen with this color/word as the src in the DMA transfer
    unsafe { ptr::write_volatile(scratchpad_ptr, 0xFF_00_00_FF) };

    let control_blocks =
        unsafe { core::slice::from_raw_parts_mut(dma_cb_pmem.vaddr as *mut ControlBlock, num_cb) };

    for ref mut cb in control_blocks.iter_mut() {
        cb.init();
    }

    // Configure a framebuffer so we can do some DMA transfers to it
    let display_width = 240;
    let display_height = 240;
    let fb_cfg = FramebufferCmd {
        phy_width: display_width,
        phy_height: display_height,
        virt_width: display_width,
        virt_height: display_height,
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

    debug_println!("Mapped in GPU memory, size = 0x{:X} bytes", mem_size_bytes);
    debug_println!(
        "  vaddr = 0x{:X} paddr = 0x{:X}",
        gpu_pmem.vaddr,
        gpu_pmem.paddr
    );

    allocator.dma_cache_op(
        gpu_pmem.vaddr,
        mem_size_bytes as _,
        DMACacheOp::CleanInvalidate,
    );

    let fb_ptr = gpu_pmem.vaddr as *mut u32;

    // Fill the screen pixel-by-pixel with green
    for y in 0..fb_resp.phy_height {
        for x in 0..fb_resp.phy_width {
            let offset = (y * (fb_resp.pitch / 4)) + x;
            unsafe { ptr::write(fb_ptr.offset(offset as _), 0xFF_00_FF_00) };
        }
    }

    // Construct a control block to black out the screen by filling the
    // framebuffer memory with a word from our scratchpad area
    let mut cb_config = ControlBlockConfig::default();

    cb_config.src_ignore = false;
    cb_config.dest_inc = true;
    cb_config.dest_width_128 = true;
    cb_config.src_width_128 = true;
    cb_config.src_inc = false;
    // only for ch0?
    cb_config.burst_length = 8;
    // TODO - testing, OUTSTANDING writes is non-zero
    cb_config.wait_for_resp = true;

    // Stride, in bytes, is a signed inc/dec applied after end of each row
    let bbp: u32 = 4;
    let stride = fb_resp.pitch - (fb_resp.phy_width * bbp);

    // This is not really obvious from the DMA documentation,
    // but the top 16 bits must be programmmed to "height -1"
    // and not "height" in 2D mode.
    cb_config.transfer_length = TransferLength::Mode2D(
        // transfer length in bytes of a row
        (bbp * fb_resp.phy_width) as _,
        // How many x-length transfers are performed
        (fb_resp.phy_height - 1) as _,
    );

    // Destination physical address of the GPU framebuffer
    let dst_paddr = fb_resp.bus_paddr;

    // Source is just a single word in our scratchpad area
    let src_paddr = scratchpad_paddr;

    // Apply control block configuration to block 0
    control_blocks[0].config(
        &cb_config,
        src_paddr,
        dst_paddr,
        stride as _,
        stride as _,
        0,
    );

    // Pick DMA channel 0
    let mut dma_channel = dma_parts.ch0;
    assert_eq!(
        dma_channel.is_lite(),
        false,
        "Can't use a lite DMA engine for 2D transfers"
    );

    debug_println!("Enabling DMA channel 0");

    dma_parts.enable.ENABLE.write(ENABLE::EN0::SET);

    dma_channel.reset();

    debug_println!("Starting DMA transfer");

    while dma_channel.is_busy() == true {}

    // Start DMA transfer, give it the paddr of our control block
    dma_channel.start(dma_cb_pmem.paddr as _);

    dma_channel.wait();

    debug_println!("{:#?}", control_blocks[0]);

    assert_eq!(dma_channel.errors(), false, "DMA errors present");

    debug_println!("All done");
}
