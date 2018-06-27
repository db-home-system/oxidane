//! Inter-Integrated Circuit (I2C) bus

use cast::u8;
use stm32l151::{I2C1, I2C2};

use gpio::gpiob::{PB10, PB11, PB6, PB7, PB8, PB9};
use gpio::AF4;
use hal::blocking::i2c::{Write, WriteRead};
use rcc::{APB1, Clocks};
use time::Hertz;

/// I2C error
#[derive(Debug)]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration loss
    Arbitration,
    // Overrun, // slave mode only
    // Pec, // SMBUS mode only
    // Timeout, // SMBUS mode only
    // Alert, // SMBUS mode only
    #[doc(hidden)]
    _Extensible,
}

// FIXME these should be "closed" traits
/// SCL pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SclPin<I2C> {}

/// SDA pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SdaPin<I2C> {}

unsafe impl SclPin<I2C1> for PB6<AF4> {}
unsafe impl SclPin<I2C1> for PB8<AF4> {}
unsafe impl SclPin<I2C2> for PB10<AF4> {}

unsafe impl SdaPin<I2C1> for PB7<AF4> {}
unsafe impl SdaPin<I2C1> for PB9<AF4> {}
unsafe impl SdaPin<I2C2> for PB11<AF4> {}

/// I2C peripheral operating in master mode
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

macro_rules! wait_sr1 {
    ($i2c:expr, $flag:ident) => {
        loop {
            let sr = $i2c.sr1.read();

            if sr.berr().bit_is_set() {
                return Err(Error::Bus);
            } else if sr.arlo().bit_is_set() {
                return Err(Error::Arbitration);
            } else if sr.$flag().bit_is_set() {
                break;
            } else {
                // try again
            }
        }
    };
}

macro_rules! hal {
    ($($I2CX:ident: ($i2cX:ident, $i2cXen:ident, $i2cXrst:ident),)+) => {
        $(
            impl<SCL, SDA> I2c<$I2CX, (SCL, SDA)> {
                /// Configures the I2C peripheral to work in master mode
                pub fn $i2cX<F>(
                    i2c: $I2CX,
                    pins: (SCL, SDA),
                    freq: F,
                    clocks: Clocks,
                    apb1: &mut APB1,
                ) -> Self where
                    F: Into<Hertz>,
                    SCL: SclPin<$I2CX>,
                    SDA: SdaPin<$I2CX>,
                {
                    apb1.enr().modify(|_, w| w.$i2cXen().set_bit());
                    apb1.rstr().modify(|_, w| w.$i2cXrst().set_bit());
                    apb1.rstr().modify(|_, w| w.$i2cXrst().clear_bit());

                    // TODO: support "fast mode"

                    let freq = freq.into().0;
                    let pclk = clocks.pclk1().0;

                    // Requirements for standard mode
                    assert!(freq <= 100_000);
                    assert!(pclk >= 2_000_000);

                    let f_range = (pclk / 1_000_000) as u8;
                    let t_rise = f_range + 1;
                    let ccr = ((pclk / (freq << 1)).max(0x04) + 1) as u16;

                    // Configure clocks
                    i2c.cr2.write(|w| unsafe {
                        w.freq().bits(f_range)
                    });

                    i2c.trise.write(|w| unsafe {
                        w.trise().bits(t_rise)
                    });

                    i2c.ccr.write(|w| unsafe {
                        w.ccr().bits(ccr)
                    });

                    // Enable the peripheral
                    i2c.cr1.write(|w| w.pe().set_bit());

                    I2c { i2c, pins }
                }

                /// Releases the I2C peripheral and associated pins
                pub fn free(self) -> ($I2CX, (SCL, SDA)) {
                    (self.i2c, self.pins)
                }
            }

            impl<PINS> Write for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
                    // START and prepare to send address
                    self.i2c.cr1.write(|w| {
                        w.start().set_bit()
                    });

                    // EV5 [RM, 26.3.3]
                    wait_sr1!(self.i2c, sb);

                    // Prepare address and wait for transmission
                    self.i2c.dr.write(|w| unsafe {
                        w.dr().bits(addr)
                    });

                    // EV6 [RM, 26.3.3]
                    wait_sr1!(self.i2c, addr);
                    self.i2c.sr2.read();

                    // Send data bytes
                    for &b in bytes {
                        self.i2c.dr.write(|w| unsafe {
                            w.dr().bits(b)
                        });

                        // EV8 [RM, 26.3.3]
                        wait_sr1!(self.i2c, tx_e);
                    }

                    // Close communication
                    self.i2c.cr1.write(|w| {
                        w.stop().set_bit()
                    });

                    Ok(())
                }
            }

            impl<PINS> WriteRead for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write_read(
                    &mut self,
                    addr: u8,
                    bytes: &[u8],
                    buffer: &mut [u8],
                ) -> Result<(), Error> {
                    Ok(())
                }
            }
        )+
    }
}

hal! {
    I2C1: (i2c1, i2c1en, i2c1rst),
    I2C2: (i2c2, i2c2en, i2c2rst),
}
