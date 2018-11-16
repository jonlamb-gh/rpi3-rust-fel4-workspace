#![no_std]

#[macro_use]
extern crate register;

const MMIO_BASE: u64 = 0x3F00_0000;

pub mod gpio;
pub mod mbox;
pub mod uart1;
