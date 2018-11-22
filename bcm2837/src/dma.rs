use super::MMIO_BASE;

use core::ops::Deref;
use register::mmio::{ReadOnly, ReadWrite};

// TODO - control block struct, alignment

// TODO - registers

/// Base address, each channel is offset by 0x100
pub const PADDR: u64 = MMIO_BASE + 0x0000_7000;

pub const CHANNEL0_OFFSET: u64 = 0x000;
pub const CHANNEL1_OFFSET: u64 = 0x100;
pub const CHANNEL2_OFFSET: u64 = 0x200;
pub const CHANNEL3_OFFSET: u64 = 0x300;
pub const CHANNEL4_OFFSET: u64 = 0x400;
pub const CHANNEL5_OFFSET: u64 = 0x500;
pub const CHANNEL6_OFFSET: u64 = 0x600;
pub const CHANNEL7_OFFSET: u64 = 0x700;
pub const CHANNEL8_OFFSET: u64 = 0x800;
pub const CHANNEL9_OFFSET: u64 = 0x900;
pub const CHANNEL10_OFFSET: u64 = 0xa00;
pub const CHANNEL11_OFFSET: u64 = 0xb00;
pub const CHANNEL12_OFFSET: u64 = 0xc00;
pub const CHANNEL13_OFFSET: u64 = 0xd00;
pub const CHANNEL14_OFFSET: u64 = 0xe00;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub CS: ReadWrite<u32>,        // 0x00
    pub CONBLK_AD: ReadWrite<u32>, // 0x04
    pub TI: ReadOnly<u32>,         // 0x08
    pub SOURCE_AD: ReadOnly<u32>,  // 0x0C
    pub DEST_AD: ReadOnly<u32>,    // 0x10
    pub TXFR_LEN: ReadOnly<u32>,   // 0x14
    pub STRIDE: ReadOnly<u32>,     // 0x18
    pub NEXTCONBK: ReadOnly<u32>,  // 0x1C
    pub DEBUG: ReadOnly<u32>,      // 0x20
}

pub struct DMA {
    addr: *const u64,
}

impl From<u64> for DMA {
    fn from(vaddr: u64) -> DMA {
        assert_ne!(vaddr, 0);
        DMA {
            addr: vaddr as *const u64,
        }
    }
}

unsafe impl Send for DMA {}

impl DMA {
    pub fn ptr(&self) -> *const RegisterBlock {
        self.addr as *const _
    }
}

impl Deref for DMA {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}
