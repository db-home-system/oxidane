//! Serial Peripheral Interface (SPI) bus

use core::ptr;

use hal::spi::{FullDuplex, Mode, Phase, Polarity};
use nb;
use stm32l151::{SPI1, SPI2};

use gpio::gpioa::{PA11, PA12, PA5, PA6, PA7};
use gpio::gpiob::{PB13, PB14, PB15, PB3, PB4, PB5};
use gpio::AF5;
use rcc::{APB1, APB2, Clocks};
use time::Hertz;

/// SPI error
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)]
    _Extensible,
}

// FIXME these should be "closed" traits
/// SCK pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SckPin<SPI> {}

/// MISO pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait MisoPin<SPI> {}

/// MOSI pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait MosiPin<SPI> {}

unsafe impl SckPin<SPI1> for PA5<AF5> {}
unsafe impl SckPin<SPI1> for PB3<AF5> {}
unsafe impl SckPin<SPI2> for PB13<AF5> {}

unsafe impl MisoPin<SPI1> for PA6<AF5> {}
unsafe impl MisoPin<SPI1> for PA11<AF5> {}
unsafe impl MisoPin<SPI1> for PB4<AF5> {}
unsafe impl MisoPin<SPI2> for PB14<AF5> {}

unsafe impl MosiPin<SPI1> for PA7<AF5> {}
unsafe impl MosiPin<SPI1> for PA12<AF5> {}
unsafe impl MosiPin<SPI1> for PB5<AF5> {}
unsafe impl MosiPin<SPI2> for PB15<AF5> {}

/// SPI peripheral operating in full duplex master mode
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

macro_rules! hal {
    ($($SPIX:ident: ($spiX:ident, $APBX:ident, $spiXen:ident, $spiXrst:ident, $pclkX:ident),)+) => {
        $(
            impl<SCK, MISO, MOSI> Spi<$SPIX, (SCK, MISO, MOSI)> {
                /// Configures the SPI peripheral to operate in full duplex master mode
                pub fn $spiX<F>(
                    spi: $SPIX,
                    pins: (SCK, MISO, MOSI),
                    mode: Mode,
                    freq: F,
                    clocks: Clocks,
                    apb2: &mut $APBX,
                ) -> Self
                where
                    F: Into<Hertz>,
                    SCK: SckPin<$SPIX>,
                    MISO: MisoPin<$SPIX>,
                    MOSI: MosiPin<$SPIX>,
                {
                    // enable or reset $SPIX
                    apb2.enr().modify(|_, w| w.$spiXen().set_bit());
                    apb2.rstr().modify(|_, w| w.$spiXrst().set_bit());
                    apb2.rstr().modify(|_, w| w.$spiXrst().clear_bit());

                    // SSOE: Slave Select output disabled
                    spi.cr2.write(|w| w.ssoe().clear_bit());

                    let br = match clocks.$pclkX().0 / freq.into().0 {
                        0 => unreachable!(),
                        1...2 => 0b000,
                        3...5 => 0b001,
                        6...11 => 0b010,
                        12...23 => 0b011,
                        24...39 => 0b100,
                        40...95 => 0b101,
                        96...191 => 0b110,
                        _ => 0b111,
                    };

                    // CPHA: phase
                    // CPOL: polarity
                    // MSTR: master mode
                    // BR: 1 MHz
                    // SPE: SPI disabled
                    // LSBFIRST: MSB first
                    // SSM: enable software slave management (NSS pin free for other uses)
                    // SSI: set nss high = master mode
                    // DFF: 8-bit data frame format
                    // CRCEN: hardware CRC calculation disabled
                    // BIDIMODE: 2 line unidirectional (full duplex)
                    spi.cr1.write(|w| unsafe {
                        w.cpha()
                            .bit(mode.phase == Phase::CaptureOnSecondTransition)
                            .cpol()
                            .bit(mode.polarity == Polarity::IdleHigh)
                            .mstr()
                            .set_bit()
                            .br()
                            .bits(br)
                            .spe()
                            .set_bit()
                            .lsbfirst()
                            .clear_bit()
                            .ssi()
                            .set_bit()
                            .ssm()
                            .set_bit()
                            .dff()
                            .clear_bit()
                            .crcen()
                            .clear_bit()
                            .bidimode()
                            .clear_bit()
                    });

                    Spi { spi, pins }
                }

                /// Releases the SPI peripheral and associated pins
                pub fn free(self) -> ($SPIX, (SCK, MISO, MOSI)) {
                    (self.spi, self.pins)
                }
            }

            impl<PINS> FullDuplex<u8> for Spi<$SPIX, PINS> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.rxne().bit_is_set() {
                        // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
                        // reading a half-word)
                        return Ok(unsafe {
                            ptr::read_volatile(&self.spi.dr as *const _ as *const u8)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }

                fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.txe().bit_is_set() {
                        // NOTE(write_volatile) see note above
                        unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
                        return Ok(());
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl<PINS> ::hal::blocking::spi::transfer::Default<u8> for Spi<$SPIX, PINS> {}

            impl<PINS> ::hal::blocking::spi::write::Default<u8> for Spi<$SPIX, PINS> {}
        )+
    }
}

hal! {
    SPI1: (spi1, APB2, spi1en, spi1rst, pclk2),
    SPI2: (spi2, APB1, spi2en, spi2rst, pclk1),
}

// FIXME not working
// TODO measure if this actually faster than the default implementation
// impl ::hal::blocking::spi::Write<u8> for Spi {
//     type Error = Error;

//     fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
//         for byte in bytes {
//             'l: loop {
//                 let sr = self.spi.sr.read();

//                 // ignore overruns because we don't care about the incoming data
//                 // if sr.ovr().bit_is_set() {
//                 // Err(nb::Error::Other(Error::Overrun))
//                 // } else
//                 if sr.modf().bit_is_set() {
//                     return Err(Error::ModeFault);
//                 } else if sr.crcerr().bit_is_set() {
//                     return Err(Error::Crc);
//                 } else if sr.txe().bit_is_set() {
//                     // NOTE(write_volatile) see note above
//                     unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, *byte) }
//                     break 'l;
//                 } else {
//                     // try again
//                 }
//             }
//         }

//         // wait until the transmission of the last byte is done
//         while self.spi.sr.read().bsy().bit_is_set() {}

//         // clear OVR flag
//         unsafe {
//             ptr::read_volatile(&self.spi.dr as *const _ as *const u8);
//         }
//         self.spi.sr.read();

//         Ok(())
//     }
// }
