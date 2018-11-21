use super::MailboxMsgBufferConstructor;
use super::Tag;
use super::MAILBOX_BUFFER_LEN;
use super::REQUEST;

pub const CMD_LEN: u32 = 8;
pub const RESP_LEN: u32 = 8;

#[derive(Debug, Copy, Clone)]
pub struct GetVcMemCmd;

#[derive(Debug, Copy, Clone)]
pub struct GetVcMemResp {
    pub addr: u32,
    pub size: u32,
}

impl MailboxMsgBufferConstructor for GetVcMemCmd {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]) {
        buffer[0] = 8 * 4;
        buffer[1] = REQUEST;
        buffer[2] = Tag::GetVcMem.into();
        buffer[3] = CMD_LEN;
        buffer[4] = RESP_LEN;
        buffer[5] = 0;
        buffer[6] = 0;
        buffer[7] = Tag::Last.into();
    }
}

impl From<&[u32; MAILBOX_BUFFER_LEN]> for GetVcMemResp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> GetVcMemResp {
        assert_eq!(buffer[2], Tag::GetVcMem.into());
        GetVcMemResp {
            addr: buffer[5],
            size: buffer[6],
        }
    }
}
