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
use bcm2837_hal::mailbox::{Mailbox, Channel};
use bcm2837_hal::mailbox_msg::{Resp, MAILBOX_BUFFER_LEN};
use bcm2837_hal::mailbox_msg::get_serial_num::GetSerialNumCmd;
use core::fmt::Write;
use sel4_sys::*;
use sel4twinkle_alloc::{Allocator, PAGE_BITS_4K, PMem};

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
    let mbox_buffer_pmem: PMem = allocator.pmem_new_page(None)
        .expect("Failed to allocate pmem");

    debug_println!("Allocated pmem page");
    debug_println!("  vaddr = 0x{:X} paddr = 0x{:X}",
        mbox_buffer_pmem.vaddr,
        mbox_buffer_pmem.paddr);

    // TODO - need to allocate some untyped/etc region of memory
    // such that I can get the paddr to give to the vc mbox core,
    // same way as with DMA

    let ptr = mbox_buffer_pmem.vaddr as *mut u64;
    let mbox_buffer_ptr = ptr as *mut [u32; MAILBOX_BUFFER_LEN];
    let mbox_buffer = unsafe { *mbox_buffer_ptr };
    //let mbox_buffer: &[u32; MAILBOX_BUFFER_LEN] = ptr as _;
    //let mbox_buffer: &[u32; MAILBOX_BUFFER_LEN] = mbox_buffer_pmem.vaddr as *const u32 as _;

    let mut mbox: Mailbox = Mailbox::new(
        MBOX::from(vc_mbox_vaddr),
        mbox_buffer_pmem.paddr as _,
        mbox_buffer,
    );

    // GPIO
    let gpio_vaddr = allocator
        .io_map(GPIO_PADDR, PAGE_BITS_4K as _)
        .expect("Failed to io_map");

    debug_println!("Mapped GPIO device region");
    debug_println!("  vaddr = 0x{:X} paddr = 0x{:X}", gpio_vaddr, GPIO_PADDR);

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

    writeln!(serial, "Mailbox send GetSerialNumCmd").ok();

    // Request serial number
    let res: Resp = mbox.call(
        Channel::Prop,
        &GetSerialNumCmd {},
    ).expect("TODO - mbox::call failed");

    writeln!(serial, "Response = {:#?}", res).ok();
}
