// TODO - need to figure out proper compiler_fence/dmb sync
// TODO - core::slice::from_raw_parts_mut()

use bcm2837::mbox::{MBOX, STATUS};
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, barrier};

use mailbox_msg::{MailboxMsgBufferConstructor, Resp, MAILBOX_BUFFER_LEN};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// The response buffer has error bit(s) set
    BadRequest,
    /// Status word was not recognized
    BadStatusWord,
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

/// Mailbox abstraction
pub struct Mailbox {
    mbox: MBOX,
    buffer_vc_paddr: u32,
    buffer: *mut MailboxBuffer,
}

#[repr(C)]
#[repr(align(16))]
struct MailboxBuffer {
    pub data: [u32; MAILBOX_BUFFER_LEN],
}

impl Mailbox {
    pub fn new(mbox: MBOX, buffer_vc_paddr: u32, buffer_vaddr: u32) -> Self {
        Self {
            mbox,
            buffer_vc_paddr,
            buffer: buffer_vaddr as *mut MailboxBuffer,
            //buffer,
        }
    }

    /// Make a mailbox call. Returns Err(MboxError) on failure, Ok(()) success
    pub fn call<T: MailboxMsgBufferConstructor>(
        &mut self,
        channel: Channel,
        constructor: &T,
    ) -> Result<Resp, Error> {
        unsafe {
            constructor.construct_buffer(&mut (*self.buffer).data);
        }

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::Release);

        // TODO - wmb() ?
        //unsafe { asm!("dmb st" : : : "memory") };

        // wait until we can write to the mailbox
        loop {
            if !self.mbox.STATUS.is_set(STATUS::FULL) {
                break;
            }

            asm::nop();
        }

        let buf_ptr = self.buffer_vc_paddr;

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
                // TODO - rmb() ?
                //unsafe { asm!("dmb ld" : : : "memory") };
                //compiler_fence(Ordering::Release);
                unsafe { barrier::dsb(barrier::SY) };

                //let status = unsafe { (*self.buffer).data[1] };
                let status: u32 =
                    unsafe { ::core::ptr::read_volatile((self.buffer as *mut u32).offset(1)) };

                // is it a valid successful response?
                return match status {
                    response_status::SUCCESS => Ok(Resp::from(unsafe { &(*self.buffer).data })),
                    response_status::ERROR => Err(Error::BadRequest),
                    _ => Err(Error::BadStatusWord),
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
