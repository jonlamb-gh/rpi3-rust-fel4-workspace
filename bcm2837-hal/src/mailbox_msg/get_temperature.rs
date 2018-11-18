use super::MailboxMsgBufferConstructor;
use super::Tag;
use super::MAILBOX_BUFFER_LEN;
use super::REQUEST;

pub const CMD_LEN: u32 = 4;
pub const RESP_LEN: u32 = 8;

#[derive(Debug, Copy, Clone)]
pub struct GetTemperatureCmd {
    pub id: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct GetTemperatureResp {
    pub id: u32,
    pub value: u32,
}

impl MailboxMsgBufferConstructor for GetTemperatureCmd {
    fn construct_buffer(&self, buffer: &mut [u32; MAILBOX_BUFFER_LEN]) {
        buffer[0] = 7 * 4;
        buffer[1] = REQUEST;
        buffer[2] = Tag::GetTemperature.into();
        buffer[3] = CMD_LEN;
        buffer[4] = RESP_LEN;
        buffer[5] = self.id;
        buffer[6] = Tag::Last.into();
    }
}

// TODO - sanity checks/result?
impl From<&[u32; MAILBOX_BUFFER_LEN]> for GetTemperatureResp {
    fn from(buffer: &[u32; MAILBOX_BUFFER_LEN]) -> GetTemperatureResp {
        assert_eq!(buffer[2], Tag::GetTemperature.into());
        //assert_eq!(buffer[3], RESP_LEN);
        GetTemperatureResp {
            id: buffer[5],
            value: buffer[6],
        }
    }
}
