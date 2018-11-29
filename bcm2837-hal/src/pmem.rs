//! Physical memory wrapper
// TODO - make generic so paddr/etc can be u32/u64

use cache::{bus_address_bits, cpu_address_bits};

pub struct PMem {
    vaddr: u64,
    paddr: u32,
    /// Size in bytes
    size: usize,
}

// as_slice()
// as_mut_slice()
// as_ptr()
// as_mut_ptr()

// bus_paddr() ?

impl PMem {
    pub fn new(vaddr: u64, paddr: u32, size: usize) -> Self {
        assert_ne!(vaddr, 0);
        assert_ne!(paddr, 0);
        assert_ne!(size, 0);
        Self { vaddr, paddr, size }
    }

    pub fn as_slice(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.size / 4) }
    }

    pub fn as_mut_slice(&self) -> &mut [u32] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size / 4) }
    }

    pub fn as_ptr(&self) -> *const u32 {
        self.vaddr as *const u32
    }

    pub fn as_mut_ptr(&self) -> *mut u32 {
        self.vaddr as *mut u32
    }

    pub fn vaddr(&self) -> u64 {
        self.vaddr
    }

    pub fn paddr(&self) -> u32 {
        self.paddr
    }

    pub fn bus_paddr(&self) -> u32 {
        self.paddr | bus_address_bits::ALIAS_4_L2_COHERENT
    }
}
