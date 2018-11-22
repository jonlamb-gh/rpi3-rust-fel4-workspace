use super::MailboxMsgBufferConstructor;
use super::Tag;
use super::MAILBOX_BUFFER_LEN;
use super::REQUEST;

pub const CMD_LEN: u32 = 16;
pub const RESP_LEN: u32 = 4;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CursorCoord {
    Display,
    Framebuffer,
}

#[derive(Debug, Copy, Clone)]
pub struct SetCursorStateCmd {
    pub enable: bool,
    pub x: u32,
    pub y: u32,
    pub coord: CursorCoord,
}

#[derive(Debug, Copy, Clone)]
pub struct SetCursorStateResp {
    pub is_valid: bool,
}

impl MailboxMsgBufferConstructor for SetCursorStateCmd {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]) {
        buffer[0] = 10 * 4;
        buffer[1] = REQUEST;
        buffer[2] = Tag::SetCursorState.into();
        buffer[3] = CMD_LEN;
        buffer[4] = RESP_LEN;
        buffer[5] = if self.enable == true { 1 } else { 0 };
        buffer[6] = self.x;
        buffer[7] = self.y;
        buffer[8] = self.coord.into();
        buffer[9] = Tag::Last.into();
    }
}

impl From<&[u32; MAILBOX_BUFFER_LEN]> for SetCursorStateResp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> SetCursorStateResp {
        assert_eq!(buffer[2], Tag::SetCursorState.into());
        SetCursorStateResp {
            is_valid: if buffer[5] == 0 { true } else { false },
        }
    }
}

impl From<CursorCoord> for u32 {
    fn from(c: CursorCoord) -> u32 {
        match c {
            CursorCoord::Display => 0,
            CursorCoord::Framebuffer => 1,
        }
    }
}
