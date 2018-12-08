//! VideoCore Mailbox

use super::MMIO_BASE;

use core::ops::Deref;
use register::mmio::{ReadOnly, WriteOnly};

register_bitfields! {
    u32,

    STATUS [
        FULL  OFFSET(31) NUMBITS(1) [],
        EMPTY OFFSET(30) NUMBITS(1) []
    ]
}

pub const BASE_PADDR: u64 = MMIO_BASE + 0xB000;
pub const BASE_OFFSET: u64 = 0x0880;
pub const PADDR: u64 = BASE_PADDR + BASE_OFFSET;

// TODO - valvers bug here? two status registers, 0x18 and 0x38
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub READ: ReadOnly<u32>,                     // 0x00
    __reserved_0: [u32; 5],                      // 0x04
    pub STATUS: ReadOnly<u32, STATUS::Register>, // 0x18
    __reserved_1: u32,                           // 0x1C
    pub WRITE: WriteOnly<u32>,                   // 0x20
    __reserved_2: [u32; 5],                      /* 0x24
                                                  *pub STATUS1: ReadOnly<u32,
                                                  * STATUS::Register>, // 0x38 */
}

#[derive(Debug, Copy, Clone)]
pub struct MBOX {
    addr: *const u64,
}

impl From<u64> for MBOX {
    fn from(vaddr: u64) -> MBOX {
        assert_ne!(vaddr, 0);
        MBOX {
            addr: vaddr as *const u64,
        }
    }
}

unsafe impl Send for MBOX {}

impl MBOX {
    pub fn as_ptr(&self) -> *const RegisterBlock {
        self.addr as *const _
    }
}

impl Deref for MBOX {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}
