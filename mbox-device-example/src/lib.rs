#![no_std]

extern crate bcm2837_hal;
extern crate sel4_sys;
extern crate sel4twinkle_alloc;

use bcm2837_hal::bcm2837::gpio::{GPIO, PADDR as GPIO_PADDR};
use bcm2837_hal::bcm2837::mbox::{
    BASE_OFFSET as MBOX_BASE_OFFSET, BASE_PADDR as MBOX_BASE_PADDR, MBOX,
};
use bcm2837_hal::bcm2837::uart1::{PADDR as UART1_PADDR, UART1};
use bcm2837_hal::serial::Serial;
use core::fmt::Write;
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
    // TODO - use mbox::...
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
        "  vaddr = 0x{:X} paddr = 0x{:X}",
        vc_mbox_vaddr,
        vc_mbox_paddr,
    );
    debug_println!(
        "  base vaddr = 0x{:X} base_paddr 0x{:X}",
        base_vaddr,
        base_paddr,
    );

    let mut _mbox = MBOX::from(vc_mbox_vaddr);

    // GPIO
    let gpio_vaddr = allocator
        .io_map(GPIO_PADDR, PAGE_BITS_4K as _)
        .expect("Failed to io_map");

    debug_println!("Mapped GPIO device region");
    debug_println!("  vaddr = 0x{:X} paddr = 0x{:X}", gpio_vaddr, GPIO_PADDR,);

    let mut gpio = GPIO::from(gpio_vaddr);

    // UART1
    let uart1_vaddr = allocator
        .io_map(UART1_PADDR, PAGE_BITS_4K as _)
        .expect("Failed to io_map");

    debug_println!("Mapped UART1 device region");
    debug_println!("  vaddr = 0x{:X} paddr = 0x{:X}", uart1_vaddr, UART1_PADDR,);

    // Serial
    let mut serial: Serial<UART1> = Serial::uart1(UART1::from(uart1_vaddr), 0, &mut gpio);

    writeln!(serial, "\nThis is output from a Serial<UART1>\n").ok();
}
