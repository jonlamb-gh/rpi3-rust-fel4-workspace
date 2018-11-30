//! DMA

// TODO
// - generate all the channels via macros
// - following https://github.com/rust-embedded/embedded-hal/issues/37#issuecomment-377823801
// - use the above to make a safe api rather than just control block addr's

// height - 1
// https://github.com/raspberrypi/linux/blob/rpi-4.19.y/drivers/video/fbdev/bcm2708_fb.c#L604
//
// DMA guide
// https://github.com/seemoo-lab/bcm-rpi3/blob/master/kernel/Documentation/DMA-API-HOWTO.txt

use bcm2837::dma::*;
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, barrier};

use cache::bus_address_bits;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransferLength {
    ModeLinear(u32),
    Mode2D(u16, u16),
}

// TODO - use a bitfield?
#[derive(Debug, Copy, Clone)]
pub struct ControlBlockConfig {
    pub int_enable: bool,
    pub transfer_length: TransferLength,
    pub wait_for_resp: bool,
    pub dest_inc: bool,
    pub dest_width_128: bool,
    pub dest_dreq: bool,
    pub dest_ignore: bool,
    pub src_inc: bool,
    pub src_width_128: bool,
    pub src_dreq: bool,
    pub src_ignore: bool,
    pub burst_length: u8,
    pub peripheral_map: u8,
    pub waits: u8,
    pub no_wide_bursts: bool,
}

impl Default for ControlBlockConfig {
    fn default() -> ControlBlockConfig {
        ControlBlockConfig {
            int_enable: false,
            transfer_length: TransferLength::ModeLinear(0),
            wait_for_resp: false,
            dest_inc: false,
            dest_width_128: false,
            dest_dreq: false,
            dest_ignore: false,
            src_inc: false,
            src_width_128: false,
            src_dreq: false,
            src_ignore: false,
            burst_length: 0,
            peripheral_map: 0,
            waits: 0,
            no_wide_bursts: false,
        }
    }
}

// TODO - this will be removed, only handling a few params
impl From<&ControlBlockConfig> for u32 {
    fn from(config: &ControlBlockConfig) -> u32 {
        let mut val: u32 = 0;

        if config.int_enable {
            val |= 1 << 0;
        }

        if let TransferLength::Mode2D(_, _) = config.transfer_length {
            val |= 1 << 1;
        }

        if config.wait_for_resp {
            val |= 1 << 3;
        }
        if config.dest_inc {
            val |= 1 << 4;
        }
        if config.dest_width_128 {
            val |= 1 << 5;
        }
        if config.dest_dreq {
            val |= 1 << 6;
        }
        if config.dest_ignore {
            val |= 1 << 7;
        }
        if config.src_inc {
            val |= 1 << 8;
        }
        if config.src_width_128 {
            val |= 1 << 9;
        }
        if config.src_dreq {
            val |= 1 << 10;
        }
        if config.src_ignore {
            val |= 1 << 11;
        }

        if config.burst_length != 0 {
            val |= (config.burst_length as u32) & 0x0F << 12;
        }

        if config.peripheral_map != 0 {
            val |= (config.peripheral_map as u32) & 0x1F << 16;
        }

        if config.waits != 0 {
            val |= (config.waits as u32) & 0x1F << 21;
        }

        if config.no_wide_bursts {
            val |= 1 << 26;
        }

        val
    }
}

pub const CONTROL_BLOCK_SIZE: usize = 8 * 4;

/// 8 words (256 bits) in length and must start at a 256-bit aligned address
#[derive(Debug)]
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

    /// src/dst - physical adddresses
    /// NOTE: the physical addresses will be translated to a bus address for
    /// the DMA engine
    pub fn config(
        &mut self,
        config: &ControlBlockConfig,
        src: u32,
        dst: u32,
        src_stride: u16,
        dst_stride: u16,
        next: u32,
    ) {
        self.info = config.into();
        self.src = src | bus_address_bits::ALIAS_4_L2_COHERENT;
        self.dst = dst | bus_address_bits::ALIAS_4_L2_COHERENT;

        match config.transfer_length {
            TransferLength::ModeLinear(l) => {
                self.set_length(l);
                self.stride = 0;
            }
            TransferLength::Mode2D(x, y) => {
                self.set_2d_mode_length(x, y);
                self.set_stride(src_stride, dst_stride);
            }
        }

        self.next = next;
        self.__reserved_0[0] = 0;
        self.__reserved_0[1] = 0;
    }

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

    pub fn reset(&self) {
        // TODO - abort first?
        self.CS.write(CS::RESET::SET);
        while self.CS.is_set(CS::RESET) == true {}
    }

    pub fn is_busy(&self) -> bool {
        // TODO - dsb(sy)?
        //unsafe { barrier::dsb(barrier::SY) };

        self.CS.is_set(CS::ACTIVE)
    }

    pub fn wait(&self) {
        // TODO - dsb(sy)?
        unsafe { barrier::dsb(barrier::SY) };

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

    /// cb_paddr - the physical address of the control block to load
    /// NOTE: the physical address will be translated to a bus address for
    /// the DMA engine
    pub fn start(&mut self, cb_paddr: u32) {
        assert_eq!(
            cb_paddr & 0x1F,
            0,
            "Control block address must be 256 bit aligned"
        );

        // TODO - dsb(sy)?
        unsafe { barrier::dsb(barrier::SY) };

        self.CONBLK_AD
            .set(cb_paddr | bus_address_bits::ALIAS_4_L2_COHERENT);
        self.CS.write(CS::ACTIVE::SET);
    }

    pub fn errors(&self) -> bool {
        if self.CS.is_set(CS::ERROR) {
            return true;
        }

        if self.DEBUG.is_set(DEBUG::READ_LAST_NOT_SET_ERROR) {
            return true;
        }

        if self.DEBUG.is_set(DEBUG::FIFO_ERROR) {
            return true;
        }

        if self.DEBUG.is_set(DEBUG::READ_ERROR) {
            return true;
        }

        if self.DEBUG.read(DEBUG::OUTSTANDING_WRITES) != 0 {
            return true;
        }

        false
    }
}
