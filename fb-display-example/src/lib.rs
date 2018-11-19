#![no_std]

extern crate bcm2837_hal;
extern crate display;
extern crate sel4_sys;
extern crate sel4twinkle_alloc;

use bcm2837_hal::bcm2837::mbox::{
    BASE_OFFSET as MBOX_BASE_OFFSET, BASE_PADDR as MBOX_BASE_PADDR, MBOX,
};
use bcm2837_hal::mailbox::Mailbox;
use bcm2837_hal::mailbox_msg::FramebufferCmd;
use display::Display;
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

    // Mailbox
    let mut mbox: Mailbox = Mailbox::new(
        MBOX::from(vc_mbox_vaddr),
        mbox_buffer_pmem.paddr as _,
        mbox_buffer_pmem.vaddr as _,
    );

    debug_println!("\nCreating a Display\n");

    let fb_cfg = FramebufferCmd {
        phy_width: 800,
        phy_height: 480,

        virt_width: 800,
        virt_height: 480,

        x_offset: 0,
        y_offset: 0,
    };

    // TODO - need to obtain a cap to the paddr returned by the videocore
    let mut display = Display::new(Some(fb_cfg), &mut mbox).expect("Display::new() failed");

    display.fill_color(0xFF00FF_u32.into());

    loop {
        // TODO
    }
}
