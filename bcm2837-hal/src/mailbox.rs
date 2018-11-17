use bcm2837::mbox::{MBOX, STATUS};
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::asm;

use mailbox_msg::{MailboxMsgBufferConstructor, Resp, MAILBOX_BUFFER_LEN};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// The response buffer has error bit(s) set
    BadRequest,
    /// Unknown error
    Unknown,
    #[doc(hidden)]
    _Extensible,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Channel {
    /// Property channel
    Prop,
    #[doc(hidden)]
    _Extensible,
}

mod response_status {
    pub const SUCCESS: u32 = 0x8000_0000;
    /// error parsing request buffer (partial response)
    pub const ERROR: u32 = 0x8000_0001;
}

#[repr(C)]
#[repr(align(16))]
pub struct Mailbox {
    mbox: MBOX,
    buffer_paddr: u32,
    buffer: [u32; MAILBOX_BUFFER_LEN],
}

impl Mailbox {
    pub fn new(mbox: MBOX, buffer_paddr: u32, buffer: [u32; MAILBOX_BUFFER_LEN]) -> Self {
        Self {
            mbox,
            buffer_paddr,
            buffer,
        }
    }

    /// Make a mailbox call. Returns Err(MboxError) on failure, Ok(()) success
    pub fn call<T: MailboxMsgBufferConstructor>(
        &mut self,
        channel: Channel,
        constructor: &T,
    ) -> Result<Resp, Error> {
        constructor.construct_buffer(&mut self.buffer);

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::Release);

        // wait until we can write to the mailbox
        loop {
            if !self.mbox.STATUS.is_set(STATUS::FULL) {
                break;
            }

            asm::nop();
        }

        let buf_ptr = self.buffer_paddr;
        //let buf_ptr = self.buffer.as_ptr() as u32;
        // TODO - need to allocate an untyped and get a paddr (same as would with DMA)?
        //panic!("TODO - need physical address?? ptr = 0x:{:X}", buf_ptr);

        // write the address of our message to the mailbox with channel identifier
        self.mbox
            .WRITE
            .set((buf_ptr & !0xF) | (u32::from(channel) & 0xF));

        // now wait for the response
        loop {
            // is there a response?
            loop {
                if !self.mbox.STATUS.is_set(STATUS::EMPTY) {
                    break;
                }

                asm::nop();
            }

            let resp: u32 = self.mbox.READ.get();

            // is it a response to our message?
            if ((resp & 0xF) == channel.into()) && ((resp & !0xF) == buf_ptr) {
                // is it a valid successful response?
                return match self.buffer[1] {
                    response_status::SUCCESS => Ok(Resp::from(&self.buffer)),
                    response_status::ERROR => Err(Error::BadRequest),
                    _ => Err(Error::Unknown),
                };
            }
        }
    }
}

impl From<Channel> for u32 {
    fn from(c: Channel) -> u32 {
        match c {
            Channel::Prop => 8,
            _ => unimplemented!(),
        }
    }
}
