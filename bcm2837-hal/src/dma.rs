//! DMA

// TODO
// - generate all the channels via macros
// - following https://github.com/rust-embedded/embedded-hal/issues/37#issuecomment-377823801
// - use the above to make a safe api rather than just control block addr's

use bcm2837::dma::*;
use cortex_a::{asm, barrier};
//use hal::prelude::*;
//use void::Void;
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering};

pub const CONTROL_BLOCK_SIZE: u32 = 8 * 4;

/// 8 words (256 bits) in length and must start at a 256-bit aligned address
#[repr(C)]
#[repr(align(32))]
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

impl ControlBlock {
    pub fn init(&mut self) {
        self.info = 0;
        self.src = 0;
        self.dst = 0;
        self.length = 0;
        self.stride = 0;
        self.next = 0;
        self.__reserved_0[0] = 0;
        self.__reserved_0[1] = 0;
    }

    // TODO - set_info(TI::Register) ?
    //pub fn set_info(&mut self, info: TI::Register) {

    pub fn set_2d_mode_length(&mut self, x_len: u16, y_len: u16) {
        // TODO - enforce/assert y_len to 14 bits
        self.length = x_len as u32 & 0x0000_FFFF;
        self.length |= (y_len as u32) << 16 & 0x3FFF_0000;
    }

    pub fn set_length(&mut self, len: u32) {
        // TODO - enforce/assert 30 bits
        self.length = len & 0x3FFF_FFFF
    }

    pub fn set_stride(&mut self, src_stride: u16, dst_stride: u16) {
        self.stride = src_stride as u32 & 0x0000_FFFF;
        self.stride |= (dst_stride as u32) << 16 & 0xFFFF_0000;
    }
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
        // TODO - dsb(sy)?
        //unsafe { barrier::dsb(barrier::SY) };

        self.CS.is_set(CS::ACTIVE)
    }

    pub fn wait(&self) {
        // TODO - dsb(sy)?
        //unsafe { barrier::dsb(barrier::SY) };

        while self.CS.is_set(CS::ACTIVE) {
            asm::nop();
        }

        // TODO
        compiler_fence(Ordering::SeqCst);
    }

    pub fn abort(&self) {
        // TODO
        unimplemented!();
    }

    pub fn start(&mut self, cb_paddr: u32) {
        // TODO - dsb(sy)?
        unsafe { barrier::dsb(barrier::SY) };

        self.CONBLK_AD.set(cb_paddr);
        self.CS.write(CS::ACTIVE::SET);
    }
}
