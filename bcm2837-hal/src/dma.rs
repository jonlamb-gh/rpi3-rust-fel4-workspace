//! DMA

// TODO
// - generate all the channels via macros
// - following https://github.com/rust-embedded/embedded-hal/issues/37#issuecomment-377823801
// - use the above to make a safe api rather than just control block addr's

use bcm2837::dma::*;
//use cortex_a::asm;
//use hal::prelude::*;
//use void::Void;
use core::ops::Deref;

/// 8 words (256 bits) in length and must start at a 256-bit aligned address
#[repr(C)]
#[repr(align(8))]
pub struct ControlBlock {
    /// Transfer information, same as TI::Register
    pub info: u32,
    /// Source address
    pub src: u32,
    /// Destination address
    pub dst: u32,
    /// Transfer length, same as TXFR_LEN::Register
    pub length: u32,
    /// 2D mode stride, same as STRIDE::Register
    pub stride: u32,
    /// Next control block address
    pub next: u32,
    #[doc(hidden)]
    __reserved_0: [u32; 2],
}

pub trait DmaExt {
    type Parts;

    fn split(self) -> Self::Parts;
}

#[derive(Debug, Copy, Clone)]
pub struct Parts {
    pub ch0: Channel,
    pub ch1: Channel,
    // ... 15
    pub int_status: IntStatusRegister,
    pub enable: EnableRegister,
}

#[derive(Debug, Copy, Clone)]
pub struct IntStatusRegister {
    addr: *const u64,
}

#[derive(Debug, Copy, Clone)]
pub struct EnableRegister {
    addr: *const u64,
}

#[derive(Debug, Copy, Clone)]
pub struct Channel {
    addr: *const u64,
}

impl Deref for Channel {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.addr as *const RegisterBlock) }
    }
}

impl Deref for EnableRegister {
    type Target = EnableRegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.addr as *const EnableRegisterBlock) }
    }
}

impl Deref for IntStatusRegister {
    type Target = IntStatusRegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.addr as *const IntStatusRegisterBlock) }
    }
}

impl DmaExt for DMA {
    type Parts = Parts;

    fn split(self) -> Parts {
        let base_vaddr = self.ptr() as u64;

        Parts {
            ch0: Channel {
                addr: (base_vaddr + CHANNEL0_OFFSET) as _,
            },
            ch1: Channel {
                addr: (base_vaddr + CHANNEL1_OFFSET) as _,
            },
            int_status: IntStatusRegister {
                addr: (base_vaddr + INT_STATUS_OFFSET) as _,
            },
            enable: EnableRegister {
                addr: (base_vaddr + ENABLE_OFFSET) as _,
            },
        }
    }
}

impl Channel {
    pub fn is_lite(&self) -> bool {
        self.DEBUG.is_set(DEBUG::LITE)
    }

    pub fn dma_id(&self) -> u8 {
        self.DEBUG.read(DEBUG::DMA_ID) as _
    }

    pub fn is_busy(&self) -> bool {
        self.CS.is_set(CS::ACTIVE)
    }

    // TODO - abort()

    // TODO - wait() with nonblock type support for busy-waits

    // TODO - start(control_block)
}
