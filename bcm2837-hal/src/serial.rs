//! Serial
//! UART0 and UART1 are significantly different so they both
//! have hand implementations rather than using a macro
//!
//! TODO - UART0

// TODO - detangle namespace
use bcm2837::gpio::*;
use bcm2837::uart1::*;
use cortex_a::asm;
use hal::prelude::*;
use hal::serial;
use void::Void;

pub struct Serial<UART> {
    uart: UART,
}

// TODO - time bits, Bps, etc
impl Serial<UART1> {
    pub fn uart1(uart: UART1, _baud_rate: u32, gpio: &mut GPIO) -> Self {
        uart.AUX_ENABLES.modify(AUX_ENABLES::MINI_UART_ENABLE::SET);
        uart.AUX_MU_IER.set(0);
        uart.AUX_MU_CNTL.set(0);
        uart.AUX_MU_LCR.write(AUX_MU_LCR::DATA_SIZE::EightBit);
        uart.AUX_MU_MCR.set(0);
        uart.AUX_MU_IER.set(0);
        uart.AUX_MU_IIR.write(AUX_MU_IIR::FIFO_CLEAR::All);
        uart.AUX_MU_BAUD.write(AUX_MU_BAUD::RATE.val(270)); // 115200 baud

        // map UART1 to GPIO pins
        gpio.GPFSEL1
            .modify(GPFSEL1::FSEL14::AF5 + GPFSEL1::FSEL15::AF5);
        //    .modify(GPFSEL1::FSEL14::TXD1 + GPFSEL1::FSEL15::RXD1);

        // Enable pins 14 and 15
        gpio.GPPUD.set(0);
        for _ in 0..150 {
            asm::nop();
        }
        gpio.GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
        for _ in 0..150 {
            asm::nop();
        }
        gpio.GPPUDCLK0.set(0);

        uart.AUX_MU_CNTL
            .write(AUX_MU_CNTL::RX_EN::Enabled + AUX_MU_CNTL::TX_EN::Enabled);

        Serial { uart }
    }

    pub fn free(self) -> UART1 {
        self.uart
    }
}

impl serial::Read<u8> for Serial<UART1> {
    // No errors
    type Error = Void;

    fn read(&mut self) -> nb::Result<u8, Void> {
        if self.uart.AUX_MU_LSR.is_set(AUX_MU_LSR::DATA_READY) {
            let mut data = self.uart.AUX_MU_IO.get() as u8;

            // convert carrige return to newline
            if data == '\r' as _ {
                data = '\n' as _;
            }

            Ok(data)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl serial::Write<u8> for Serial<UART1> {
    // No errors
    type Error = Void;

    fn flush(&mut self) -> nb::Result<(), Void> {
        if self.uart.AUX_MU_LSR.is_set(AUX_MU_LSR::TX_EMPTY) {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Void> {
        if self.uart.AUX_MU_LSR.is_set(AUX_MU_LSR::TX_EMPTY) {
            self.uart.AUX_MU_IO.set(byte as _);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl ::core::fmt::Write for Serial<UART1> {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for b in s.bytes() {
            // convert '\n' to '\r\n'
            if b as char == '\n' {
                block!(self.write('\r' as _)).ok();
            }
            block!(self.write(b)).ok();
        }
        Ok(())
    }
}
