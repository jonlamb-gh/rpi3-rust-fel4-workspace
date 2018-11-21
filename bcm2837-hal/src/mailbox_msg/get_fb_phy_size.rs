use super::MailboxMsgBufferConstructor;
use super::Tag;
use super::MAILBOX_BUFFER_LEN;
use super::REQUEST;

pub const CMD_LEN: u32 = 8;
pub const RESP_LEN: u32 = 8;

#[derive(Debug, Copy, Clone)]
pub struct GetFbPhySizeCmd;

#[derive(Debug, Copy, Clone)]
pub struct GetFbPhySizeResp {
    pub width: u32,
    pub height: u32,
}

impl MailboxMsgBufferConstructor for GetFbPhySizeCmd {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]) {
        buffer[0] = 8 * 4;
        buffer[1] = REQUEST;
        buffer[2] = Tag::GetPhySize.into();
        buffer[3] = CMD_LEN;
        buffer[4] = RESP_LEN;
        buffer[5] = 0;
        buffer[6] = 0;
        buffer[7] = Tag::Last.into();
    }
}

impl From<&[u32; MAILBOX_BUFFER_LEN]> for GetFbPhySizeResp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> GetFbPhySizeResp {
        assert_eq!(buffer[2], Tag::GetPhySize.into());
        GetFbPhySizeResp {
            width: buffer[5],
            height: buffer[6],
        }
    }
}
