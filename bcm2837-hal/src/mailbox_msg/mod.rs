mod blank_screen;
mod framebuffer;
mod get_arm_mem;
mod get_fb_phy_size;
mod get_serial_num;
mod get_temperature;
mod get_vc_mem;
mod set_cursor_state;

pub use self::blank_screen::{BlankScreenCmd, BlankScreenResp};
pub use self::framebuffer::{FramebufferCmd, FramebufferResp};
pub use self::get_arm_mem::{GetArmMemCmd, GetArmMemResp};
pub use self::get_fb_phy_size::{GetFbPhySizeCmd, GetFbPhySizeResp};
pub use self::get_serial_num::{GetSerialNumCmd, GetSerialNumResp};
pub use self::get_temperature::{GetTemperatureCmd, GetTemperatureResp};
pub use self::get_vc_mem::{GetVcMemCmd, GetVcMemResp};
pub use self::set_cursor_state::{CursorCoord, SetCursorStateCmd, SetCursorStateResp};

pub const REQUEST: u32 = 0;
pub const MAILBOX_BUFFER_LEN: usize = 36;

pub trait MailboxMsgBufferConstructor {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]);
}

// TODO - compare with linux rpi firmware source for naming
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tag {
    Last,
    SetCursorState,
    GetSerialNum,
    GetArmMem,
    GetVcMem,
    GetTemperature,
    AllocBuffer,
    BlankScreen,
    GetPitch,
    GetPhySize,
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
            Tag::SetCursorState => 0x8011,
            Tag::GetSerialNum => 0x10004,
            Tag::GetArmMem => 0x10005,
            Tag::GetVcMem => 0x10006,
            Tag::GetTemperature => 0x30006,
            Tag::AllocBuffer => 0x40001,
            Tag::BlankScreen => 0x40002,
            Tag::GetPhySize => 0x40003,
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
    GetArmMemResp(GetArmMemResp),
    GetVcMemResp(GetVcMemResp),
    BlankScreenResp(BlankScreenResp),
    GetFbPhySizeResp(GetFbPhySizeResp),
    SetCursorStateResp(SetCursorStateResp),
}

// TODO - sanity checks/result?
impl From<&[u32; MAILBOX_BUFFER_LEN]> for Resp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> Resp {
        // TODO - would prefer to match
        if buffer[2] == Tag::GetSerialNum.into() {
            Resp::GetSerialNumResp(GetSerialNumResp::from(buffer))
        } else if buffer[2] == Tag::GetTemperature.into() {
            Resp::GetTemperatureResp(GetTemperatureResp::from(buffer))
        } else if buffer[2] == Tag::GetArmMem.into() {
            Resp::GetArmMemResp(GetArmMemResp::from(buffer))
        } else if buffer[2] == Tag::GetVcMem.into() {
            Resp::GetVcMemResp(GetVcMemResp::from(buffer))
        } else if buffer[2] == Tag::BlankScreen.into() {
            Resp::BlankScreenResp(BlankScreenResp::from(buffer))
        } else if buffer[2] == Tag::GetPhySize.into() {
            Resp::GetFbPhySizeResp(GetFbPhySizeResp::from(buffer))
        } else if buffer[2] == Tag::SetCursorState.into() {
            Resp::SetCursorStateResp(SetCursorStateResp::from(buffer))
        } else if buffer[2] == Tag::SetPhySize.into() {
            // TODO - gating on the first tag, need to improve this
            Resp::FramebufferResp(FramebufferResp::from(buffer))
        } else {
            Resp::Ack
        }
    }
}
