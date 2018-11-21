use super::MailboxMsgBufferConstructor;
use super::Tag;
use super::MAILBOX_BUFFER_LEN;
use super::REQUEST;

pub const CMD_LEN: u32 = 8;
pub const RESP_LEN: u32 = 8;

#[derive(Debug, Copy, Clone)]
pub struct BlankScreenCmd {
    pub state: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct BlankScreenResp {
    pub state: bool,
}

impl MailboxMsgBufferConstructor for BlankScreenCmd {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]) {
        buffer[0] = 8 * 4;
        buffer[1] = REQUEST;
        buffer[2] = Tag::BlankScreen.into();
        buffer[3] = CMD_LEN;
        buffer[4] = RESP_LEN;
        buffer[5] = if self.state { 1 } else { 0 };
        buffer[6] = 0;
        buffer[7] = Tag::Last.into();
    }
}

// TODO - sanity checks/result?
impl From<&[u32; MAILBOX_BUFFER_LEN]> for BlankScreenResp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> BlankScreenResp {
        assert_eq!(buffer[2], Tag::BlankScreen.into());
        //assert_eq!(buffer[3], RESP_LEN);
        BlankScreenResp {
            state: if buffer[5] == 0 { false } else { true },
        }
    }
}
