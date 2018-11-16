#![no_std]

extern crate sel4_sys;
extern crate sel4twinkle_alloc;
extern crate bcm2837_hal;

use sel4_sys::*;
use sel4twinkle_alloc::{Allocator, PAGE_BITS_4K};

#[macro_use]
mod macros;

pub fn handle_fault(badge: seL4_Word) {
    debug_println!("\n!!! Fault from badge 0x{:X}\n", badge);
}

pub fn init(allocator: &mut Allocator, _global_fault_ep_cap: seL4_CPtr) {
    debug_println!("\nHello from custom init fn\n");

    // VideoCore Mailbox is at 0x3F00_B880
    let base_size = PAGE_BITS_4K as usize;
    let base_paddr: seL4_Word = 0x3F00_B000;
    let vc_mbox_offset: seL4_Word = 0x880;
    let vc_mbox_paddr: seL4_Word = base_paddr + vc_mbox_offset;

    let base_vaddr = allocator
        .io_map(base_paddr, base_size)
        .expect("Failed to io_map");

    let vc_mbox_vaddr = base_vaddr + vc_mbox_offset;

    debug_println!("Mapped VideoCore Mailbox device region");
    debug_println!(
        "vaddr = 0x{:X} paddr = 0x{:X}",
        vc_mbox_vaddr,
        vc_mbox_paddr,
    );
    debug_println!(
        "base vaddr = 0x{:X} base_paddr 0x{:X}",
        base_vaddr,
        base_paddr,
    );
}
