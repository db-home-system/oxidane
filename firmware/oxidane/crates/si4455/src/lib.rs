#![no_std]

extern crate embedded_hal as hal;
extern crate generic_array;

mod defs;
pub use defs::*;

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
    /// Creates a new instance of the radio device.
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

        /* Send POWER_UP (black magic) sequence */
        si4455.transfer(
            &[Command::POWER_UP as u8, 0x01, 0x00, 0x01, 0xC9, 0xC3, 0x80],
            &mut [0],
        )?;

        Ok(si4455)
    }

    /// Reports basic information about the device.
    pub fn get_part_info(&mut self) -> Result<PartInfo, E> {
        let cmd = [Command::PART_INFO as u8];
        let mut resp = [0; 9];

        self.transfer(&cmd, &mut resp)?;

        Ok(PartInfo {
            revision: resp[0],
            part: (resp[1] as u16) << 8 | resp[2] as u16,
            builder: resp[3],
            id: (resp[4] as u16) << 8 | resp[5] as u16,
            customer: resp[6],
            rom_id: resp[7],
            bond: resp[8],
        })
    }

    /// Reports function revision information about the device.
    pub fn get_func_info(&mut self) -> Result<FuncInfo, E> {
        let cmd = [Command::FUNC_INFO as u8];
        let mut resp = [0; 11];

        self.transfer(&cmd, &mut resp)?;

        Ok(FuncInfo {
            rev_ext: resp[0],
            rev_branch: resp[1],
            rev_int: resp[2],
            patch: (resp[3] as u16) << 8 | resp[4] as u16,
            func: resp[5],
            svn_flags: resp[6],
            svn_rev: (resp[7] as u32) << 24
                | (resp[8] as u32) << 16
                | (resp[9] as u32) << 8
                | resp[10] as u32,
        })
    }

    /// Blocks until the radio is ready to receive a new command.
    fn wait_for_cts(&mut self) -> Result<(), E> {
        // Send a dummy command and wait for the response, it means the radio is ready
        self.read(&mut [0_u8])
    }

    /// Low-level method to send a buffer to the radio.
    fn write(&mut self, tx: &[u8]) -> Result<(), E> {
        // Wait for the radio to be ready before sending stuff
        self.wait_for_cts()?;

        self.ncs.set_low();
        self.spi.write(tx)?;
        self.ncs.set_high();

        Ok(())
    }

    /// Low-level method to read a chunk of data from the radio
    fn read(&mut self, rx: &mut [u8]) -> Result<(), E> {
        // TODO: some sort of timeout should be used?
        loop {
            let mut scratch = [Command::READ_CMD_BUFF as u8, 0x00];

            self.ncs.set_low();
            self.spi.transfer(&mut scratch)?;

            if scratch[1] == CTS_READY {
                self.spi.transfer(rx)?;
                self.ncs.set_high();
                return Ok(());
            }

            self.ncs.set_high();
            // TODO: is it necessary to put a delay here?
        }
    }

    /// Transfers a command buffer to the radio and receives the response in rx
    fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), E> {
        self.write(tx)?;
        self.read(rx)
    }
}

// SPI mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

// Clear-to-send
const CTS_READY: u8 = 0xFF;

// Radio commands
#[allow(non_camel_case_types)]
enum Command {
    PART_INFO = 0x01,
    POWER_UP = 0x02,
    FUNC_INFO = 0x10,
    READ_CMD_BUFF = 0x44,
}
