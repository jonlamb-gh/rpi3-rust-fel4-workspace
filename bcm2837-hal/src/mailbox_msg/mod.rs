mod get_serial_num;
mod get_temperature;

pub use self::get_serial_num::{GetSerialNumCmd, GetSerialNumResp};
pub use self::get_temperature::{GetTemperatureCmd, GetTemperatureResp};

pub const REQUEST: u32 = 0;
pub const MAILBOX_BUFFER_LEN: usize = 36;

pub trait MailboxMsgBufferConstructor {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tag {
    Last,
    GetSerialNum,
    GetTemperature,
    #[doc(hidden)]
    _Extensible,
}

impl From<Tag> for u32 {
    fn from(tag: Tag) -> u32 {
        match tag {
            Tag::Last => 0,
            Tag::GetSerialNum => 0x0001_0004,
            Tag::GetTemperature => 0x0003_0006,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Resp {
    Ack,
    GetSerialNumResp(GetSerialNumResp),
    GetTemperatureResp(GetTemperatureResp),
}

// TODO - sanity checks/result?
impl From<&[u32; MAILBOX_BUFFER_LEN]> for Resp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> Resp {
        // TODO - would prefer to match
        if buffer[2] == Tag::GetSerialNum.into() {
            Resp::GetSerialNumResp(GetSerialNumResp::from(buffer))
        } else if buffer[2] == Tag::GetTemperature.into() {
            Resp::GetTemperatureResp(GetTemperatureResp::from(buffer))
        } else {
            Resp::Ack
        }
    }
}
