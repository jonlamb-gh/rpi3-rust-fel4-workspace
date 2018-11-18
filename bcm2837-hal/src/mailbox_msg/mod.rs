mod framebuffer;
mod get_serial_num;
mod get_temperature;

pub use self::framebuffer::{FramebufferCmd, FramebufferResp};
pub use self::get_serial_num::{GetSerialNumCmd, GetSerialNumResp};
pub use self::get_temperature::{GetTemperatureCmd, GetTemperatureResp};

pub const REQUEST: u32 = 0;
pub const MAILBOX_BUFFER_LEN: usize = 36;

pub trait MailboxMsgBufferConstructor {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]);
}

// TODO - compare with linux rpi firmware source for naming
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tag {
    Last,
    GetSerialNum,
    GetTemperature,
    AllocBuffer,
    GetPitch,
    SetPhySize,
    SetVirtSize,
    SetDepth,
    SetPixelOrder,
    SetVirtOffset,
    #[doc(hidden)]
    _Extensible,
}

impl From<Tag> for u32 {
    fn from(tag: Tag) -> u32 {
        match tag {
            Tag::Last => 0,
            Tag::GetSerialNum => 0x10004,
            Tag::GetTemperature => 0x30006,
            Tag::AllocBuffer => 0x40001,
            Tag::GetPitch => 0x40008,
            Tag::SetPhySize => 0x48003,
            Tag::SetVirtSize => 0x48004,
            Tag::SetDepth => 0x48005,
            Tag::SetPixelOrder => 0x48006,
            Tag::SetVirtOffset => 0x48009,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Resp {
    Ack,
    GetSerialNumResp(GetSerialNumResp),
    GetTemperatureResp(GetTemperatureResp),
    FramebufferResp(FramebufferResp),
}

// TODO - sanity checks/result?
impl From<&[u32; MAILBOX_BUFFER_LEN]> for Resp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> Resp {
        // TODO - would prefer to match
        if buffer[2] == Tag::GetSerialNum.into() {
            Resp::GetSerialNumResp(GetSerialNumResp::from(buffer))
        } else if buffer[2] == Tag::GetTemperature.into() {
            Resp::GetTemperatureResp(GetTemperatureResp::from(buffer))
        } else if buffer[2] == Tag::SetPhySize.into() {
            // TODO - gating on the first tag, need to improve this
            Resp::FramebufferResp(FramebufferResp::from(buffer))
        } else {
            Resp::Ack
        }
    }
}
