#![no_std]

extern crate embedded_hal as hal;

use core::any::Any;
use core::marker::PhantomData;

use hal::blocking::delay::DelayMs;
use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::{Mode, Phase, Polarity};

pub struct Si4455<SPI, NCS, SDN> {
    spi: SPI,
    ncs: NCS,
    sdn: SDN,
}

impl<E, SPI, NCS, SDN> Si4455<SPI, NCS, SDN>
where
    SPI: spi::Write<u8, Error = E> + spi::Transfer<u8, Error = E>,
    NCS: OutputPin,
    SDN: OutputPin,
{
    pub fn new<D>(spi: SPI, ncs: NCS, sdn: SDN, delay: &mut D) -> Result<Si4455<SPI, NCS, SDN>, E>
    where
        D: DelayMs<u8>,
    {
        let mut si4455 = Si4455 { spi, ncs, sdn };

        /* Reset the device */
        si4455.sdn.set_high();
        delay.delay_ms(1);
        si4455.sdn.set_low();

        Ok(si4455)
    }
}

// SPI mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnSecondTransition,
};
