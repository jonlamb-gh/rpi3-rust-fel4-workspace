use super::MailboxMsgBufferConstructor;
use super::Tag;
use super::MAILBOX_BUFFER_LEN;
use super::REQUEST;

pub const CMD_LEN: u32 = 0;
pub const RESP_LEN: u32 = 8;

#[derive(Debug, Copy, Clone)]
pub struct GetSerialNumCmd;

#[derive(Debug, Copy, Clone)]
pub struct GetSerialNumResp {
    pub serial_number: u64,
}

impl MailboxMsgBufferConstructor for GetSerialNumCmd {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]) {
        buffer[0] = 6 * 4;
        buffer[1] = REQUEST;
        buffer[2] = Tag::GetSerialNum.into();
        buffer[3] = CMD_LEN;
        buffer[4] = RESP_LEN;
        buffer[5] = Tag::Last.into();
    }
}

// TODO - sanity checks/result?
impl From<&[u32; MAILBOX_BUFFER_LEN]> for GetSerialNumResp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> GetSerialNumResp {
        assert_eq!(buffer[2], Tag::GetSerialNum.into());
        //assert_eq!(buffer[3], RESP_LEN);
        GetSerialNumResp {
            serial_number: buffer[5] as u64 | (buffer[6] as u64) << 32,
        }
    }
}
