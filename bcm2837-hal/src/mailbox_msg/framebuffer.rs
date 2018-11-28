use super::super::cache::{bus_address_bits, cpu_address_bits};
use super::MailboxMsgBufferConstructor;
use super::Tag;
use super::MAILBOX_BUFFER_LEN;
use super::REQUEST;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PixelOrder {
    BGR,
    RGB,
}

#[derive(Debug, Copy, Clone)]
pub struct FramebufferCmd {
    pub phy_width: u32,
    pub phy_height: u32,
    pub virt_width: u32,
    pub virt_height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct FramebufferResp {
    // TODO - what else is useful?
    pub phy_width: u32,
    pub phy_height: u32,
    pub pitch: u32,
    pub pixel_order: PixelOrder,
    pub bus_paddr: u32,
    pub paddr: u32,
}

impl MailboxMsgBufferConstructor for FramebufferCmd {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]) {
        buffer[0] = 35 * 4;
        buffer[1] = REQUEST;

        buffer[2] = Tag::SetPhySize.into();
        buffer[3] = 8;
        buffer[4] = 8;
        buffer[5] = self.phy_width;
        buffer[6] = self.phy_height;

        buffer[7] = Tag::SetVirtSize.into();
        buffer[8] = 8;
        buffer[9] = 8;
        buffer[10] = self.virt_width;
        buffer[11] = self.virt_height;

        buffer[12] = Tag::SetVirtOffset.into();
        buffer[13] = 8;
        buffer[14] = 8;
        buffer[15] = self.x_offset;
        buffer[16] = self.y_offset;

        buffer[17] = Tag::SetDepth.into();
        buffer[18] = 4;
        buffer[19] = 4;
        buffer[20] = 32;

        buffer[21] = Tag::SetPixelOrder.into();
        buffer[22] = 4;
        buffer[23] = 4;
        buffer[24] = PixelOrder::RGB.into();

        buffer[25] = Tag::AllocBuffer.into();
        buffer[26] = 8;
        buffer[27] = 8;
        buffer[28] = 4096;
        buffer[29] = 0;

        buffer[30] = Tag::GetPitch.into();
        buffer[31] = 4;
        buffer[32] = 4;
        buffer[33] = 0;

        buffer[34] = Tag::Last.into();
    }
}

// TODO - sanity checks/result?
impl From<&[u32; MAILBOX_BUFFER_LEN]> for FramebufferResp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> FramebufferResp {
        // depth
        assert_eq!(buffer[20], 32);
        // buffer
        assert_ne!(buffer[28], 0);

        // Make sure bus address bits are correct, for QEMU mostly
        let bus_paddr = buffer[28] | bus_address_bits::ALIAS_4_L2_COHERENT;

        FramebufferResp {
            phy_width: buffer[5],
            phy_height: buffer[6],
            pitch: buffer[33],
            pixel_order: buffer[24].into(),
            bus_paddr,
            paddr: bus_paddr & cpu_address_bits::MASK,
        }
    }
}

impl From<PixelOrder> for u32 {
    fn from(po: PixelOrder) -> u32 {
        match po {
            PixelOrder::BGR => 0,
            PixelOrder::RGB => 1,
        }
    }
}

impl From<u32> for PixelOrder {
    fn from(val: u32) -> PixelOrder {
        match val {
            0 => PixelOrder::BGR,
            1 => PixelOrder::RGB,
            _ => unimplemented!(),
        }
    }
}
