#![no_std]

extern crate bcm2837_hal;
extern crate sel4_sys;
extern crate sel4twinkle_alloc;

use bcm2837_hal::bcm2837::gpio::{GPIO, PADDR as GPIO_PADDR};
use bcm2837_hal::bcm2837::mbox::{
    BASE_OFFSET as MBOX_BASE_OFFSET, BASE_PADDR as MBOX_BASE_PADDR, MBOX,
};
use bcm2837_hal::bcm2837::uart1::{PADDR as UART1_PADDR, UART1};
use bcm2837_hal::mailbox::{Channel, Mailbox};
use bcm2837_hal::mailbox_msg::*;
use bcm2837_hal::serial::Serial;
use core::fmt::Write;
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
        mbox_buffer_pmem.paddr as _,
        mbox_buffer_pmem.vaddr as _,
    );

    writeln!(serial, "\nMailbox send GetSerialNumCmd\n").ok();
    let res: Resp = mbox
        .call(Channel::Prop, &GetSerialNumCmd)
        .expect("Mailbox::call failed");
    writeln!(serial, "Response = {:#?}", res).ok();

    writeln!(serial, "\nMailbox send GetTemperatureCmd\n").ok();
    let res: Resp = mbox
        .call(Channel::Prop, &GetTemperatureCmd { id: 0 })
        .expect("Mailbox::call failed");
    writeln!(serial, "Response = {:#?}", res).ok();

    writeln!(serial, "\nMailbox send GetArmMemCmd\n").ok();
    let res: Resp = mbox
        .call(Channel::Prop, &GetArmMemCmd)
        .expect("Mailbox::call failed");
    writeln!(serial, "Response = {:#?}", res).ok();

    writeln!(serial, "\nMailbox send GetVcMemCmd\n").ok();
    let res: Resp = mbox
        .call(Channel::Prop, &GetVcMemCmd)
        .expect("Mailbox::call failed");
    writeln!(serial, "Response = {:#?}", res).ok();

    writeln!(serial, "\nMailbox send GetFbPhySizeCmd\n").ok();
    let res: Resp = mbox
        .call(Channel::Prop, &GetFbPhySizeCmd)
        .expect("Mailbox::call failed");
    writeln!(serial, "Response = {:#?}", res).ok();

    writeln!(serial, "\nMailbox send FramebufferCmd\n").ok();
    let fb_res: Resp = mbox
        .call(
            Channel::Prop,
            &FramebufferCmd {
                phy_width: 240,
                phy_height: 240,
                virt_width: 240,
                virt_height: 240,
                x_offset: 0,
                y_offset: 0,
            },
        ).expect("Mailbox::call failed");
    writeln!(serial, "Response = {:#?}", fb_res).ok();

    writeln!(serial, "\nAll done").ok();
}
