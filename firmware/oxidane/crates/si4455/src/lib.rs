#![no_std]

extern crate embedded_hal as hal;
extern crate generic_array;

use core::any::Any;
use core::marker::PhantomData;

use generic_array::typenum::consts::*;
use generic_array::{ArrayLength, GenericArray};
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
    /// Creates a new instance of the radio device
    pub fn new<D>(spi: SPI, ncs: NCS, sdn: SDN, delay: &mut D) -> Result<Si4455<SPI, NCS, SDN>, E>
    where
        D: DelayMs<u8>,
    {
        let mut si4455 = Si4455 { spi, ncs, sdn };

        /* Reset the device */
        si4455.sdn.set_high();
        delay.delay_ms(1);
        si4455.sdn.set_low();

        /* Poll until ready */
        si4455.wait_for_cts()?;

        Ok(si4455)
    }

    /// Obtains information about the chip
    pub fn get_part_info(&mut self) -> Result<[u8; 9], E> {
        let mut buffer = [0; 9];

        self.wait_for_cts()?;
        self.write(Command::PART_INFO)?;
        self.read_many(&mut buffer)?;
        Ok(buffer)
    }

    /// Blocks until the radio is ready to receive a new command
    fn wait_for_cts(&mut self) -> Result<(), E> {
        loop {
            self.write(Command::READ_CMD_BUFF)?;
            if self.read()? == CTS {
                return Ok(());
            }
        }
    }

    /// Sends a command to the radio
    fn write(&mut self, cmd: Command) -> Result<(), E> {
        self.ncs.set_low();
        self.spi.write(&[cmd as u8])?;
        self.ncs.set_high();
        Ok(())
    }

    /// Reads a single byte from the radio
    fn read(&mut self) -> Result<u8, E> {
        let mut buffer = [0; 1];
        self.read_many(&mut buffer)?;
        Ok(buffer[0])
    }

    /// Reads a stream of bytes into the provided buffer
    fn read_many(&mut self, buf: &mut [u8]) -> Result<(), E> {
        self.ncs.set_low();
        self.spi.transfer(buf)?;
        self.ncs.set_high();
        Ok(())
    }
}

// SPI mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnSecondTransition,
};

// Clear-to-send
const CTS: u8 = 0xFF;

// Radio commands
enum Command {
    PART_INFO = 0x01,
    READ_CMD_BUFF = 0x44,
}
