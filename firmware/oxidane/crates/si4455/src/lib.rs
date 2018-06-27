#![no_std]

extern crate embedded_hal as hal;
extern crate generic_array;

mod defs;
pub use defs::*;

use hal::blocking::delay::DelayMs;
use hal::blocking::spi;
use hal::digital::{InputPin, OutputPin};
use hal::spi::{Mode, Phase, Polarity};

#[derive(Debug)]
pub enum Error<E> {
    CommandError,
    Busy,
    Spi(E),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::Spi(error)
    }
}

pub struct Si4455<SPI, NCS, SDN, NIRQ> {
    spi: SPI,
    ncs: NCS,
    sdn: SDN,
    nirq: NIRQ,
}

impl<E, SPI, NCS, SDN, NIRQ> Si4455<SPI, NCS, SDN, NIRQ>
where
    SPI: spi::Write<u8, Error = E> + spi::Transfer<u8, Error = E>,
    NCS: OutputPin,
    SDN: OutputPin,
    NIRQ: InputPin,
{
    /// Creates a new instance of the radio device.
    pub fn new<D>(
        spi: SPI,
        ncs: NCS,
        sdn: SDN,
        nirq: NIRQ,
        delay: &mut D,
        config: &[u8],
    ) -> Result<Si4455<SPI, NCS, SDN, NIRQ>, Error<E>>
    where
        D: DelayMs<u8>,
    {
        let mut si4455 = Si4455 {
            spi,
            ncs,
            sdn,
            nirq,
        };

        // Perform the initial reset
        si4455.reset(delay)?;

        // Device initialization
        si4455.initialize(config)?;

        Ok(si4455)
    }

    /// Reports basic information about the device.
    pub fn get_part_info(&mut self) -> Result<PartInfo, Error<E>> {
        let mut resp = [0; 9];

        self.transfer(Command::PART_INFO as u8, &[], &mut resp)?;

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
    pub fn get_func_info(&mut self) -> Result<FuncInfo, Error<E>> {
        let mut resp = [0; 11];

        self.transfer(Command::FUNC_INFO as u8, &[], &mut resp)?;

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

    /// Retrieves and clears the radio's interrupt status register.
    fn get_int_status(&mut self) -> Result<IntStatus, Error<E>> {
        let mut resp = [0; 8];

        // The three zero arguments will clear all the pending interrupts
        self.transfer(Command::GET_INT_STATUS as u8, &[0, 0, 0], &mut resp)?;

        Ok(IntStatus {
            int_pending: resp[0],
            int_status: resp[1],
            ph_pending: resp[2],
            ph_status: resp[3],
            modem_pending: resp[4],
            modem_status: resp[5],
            chip_pending: resp[6],
            chip_status: resp[7],
        })
    }

    /// Retrieves the current device state.
    pub fn state(&mut self) -> Result<DeviceState, Error<E>> {
        let mut resp = [0; 2];

        self.transfer(Command::REQUEST_DEVICE_STATE as u8, &[], &mut resp)?;

        Ok(DeviceState {
            state: resp[0],
            channel: resp[1],
        })
    }

    /// Starts transmission of a packet.
    ///
    /// Note: this function does not wait for the transmission to finish.
    pub fn transmit(&mut self, channel: u8, packet: &[u8]) -> Result<(), Error<E>> {
        if packet.len() == 0 {
            return Ok(());
        }

        // Clear pending interrupts
        self.get_int_status()?;

        // Wait for the device to finish the previous transmission
        let mut retries = 0xF000;

        while retries > 0 {
            let state = self.state()?.state;

            if (state & State::Tx as u8) == 0 && (state & State::TxTune as u8) == 0 {
                break;
            } else {
                retries -= 1;
            }
        }

        // If the device is still busy exit with an error
        if retries == 0 {
            return Err(Error::Busy);
        }

        // Write to TX FIFO
        self.write(Command::WRITE_TX_FIFO as u8, packet)?;

        // Start transfer
        self.write(
            Command::START_TX as u8,
            &[
                channel,
                0x80, // condition: return to RX after sending
                (packet.len() >> 8) as u8,
                packet.len() as u8,
                0,
            ],
        )
    }

    /// Puts the radio in RX mode, listening for new packets.
    pub fn listen(&mut self, channel: u8, length: u16) -> Result<(), Error<E>> {
        // Clear pending interrupts
        self.get_int_status()?;

        self.write(
            Command::START_RX as u8,
            &[
                channel,
                0, // conditioning
                (length >> 8) as u8,
                length as u8,
                0x8, // nextState1
                0x8, // nextState2
                0x8, // nextState3
            ],
        )
    }

    /// Resets the radio to its initial state [AN692, §4.4].
    fn reset<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayMs<u8>,
    {
        // Reset the device
        self.sdn.set_high();
        delay.delay_ms(1);
        self.sdn.set_low();

        // Wait for POR
        delay.delay_ms(5);
        self.wait_for_cts()?;

        Ok(())
    }

    /// Initializes the device using the provided configuration array.
    fn initialize(&mut self, mut config: &[u8]) -> Result<(), Error<E>> {
        let mut resp = [0x00];

        // Send all configuration strings
        while config[0] != 0x00 {
            let len = config[0] as usize;
            let cmd = &config[1..len + 1];
            config = &config[len + 1..];

            // Special handling for messages with length >= 16
            if len > 16 {
                if cmd[0] == Command::WRITE_TX_FIFO as u8 {
                    if len > 128 {
                        // WRITE_TX_FIFO cannot exceed 128 bytes!
                        return Err(Error::CommandError);
                    }

                    // Send EZConfigArray by simply using a write
                    self.write(cmd[0], &cmd[1..])?;
                    continue;
                } else {
                    // Only WRITE_TX_FIFO can exceed 16 bytes in length!
                    return Err(Error::CommandError);
                }
            }

            // Send command and check response
            self.transfer(cmd[0], &cmd[1..], &mut resp)?;

            // If command is EZCONFIG_CHECK, we need to check that the response is non-zero
            if cmd[0] == Command::EZCONFIG_CHECK as u8 && resp[0] != 0 {
                return Err(Error::CommandError);
            }

            // Check if any error is detected using the interrupt line
            if self.nirq.is_low() {
                let ints = self.get_int_status()?;

                if ints.chip_pending & 0x08 != 0 {
                    return Err(Error::CommandError);
                }
            }
        }

        Ok(())
    }

    /// Blocks until the radio is ready to receive a new command.
    fn wait_for_cts(&mut self) -> Result<(), Error<E>> {
        // Send a NOP command and wait for the response, it means the radio is ready
        self.read(&mut [Command::NOP as u8])
    }

    /// Low-level method to send a command to the radio.
    fn write(&mut self, cmd: u8, args: &[u8]) -> Result<(), Error<E>> {
        // Wait for the radio to be ready before sending stuff
        self.wait_for_cts()?;

        self.ncs.set_low();
        self.spi.write(&[cmd])?;
        if args.len() > 0 {
            self.spi.write(args)?;
        }
        self.ncs.set_high();

        Ok(())
    }

    /// Low-level method to read a chunk of data from the radio
    fn read(&mut self, rx: &mut [u8]) -> Result<(), Error<E>> {
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

    /// Sends a command and its arguments to the radio and receives the response in rx
    fn transfer(&mut self, cmd: u8, args: &[u8], resp: &mut [u8]) -> Result<(), Error<E>> {
        self.write(cmd, args)?;
        self.read(resp)
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
#[allow(unused)]
#[allow(non_camel_case_types)]
enum Command {
    NOP = 0x00,
    PART_INFO = 0x01,
    POWER_UP = 0x02,
    FUNC_INFO = 0x10,
    EZCONFIG_CHECK = 0x19,
    GET_INT_STATUS = 0x20,
    START_TX = 0x31,
    START_RX = 0x32,
    REQUEST_DEVICE_STATE = 0x33,
    READ_CMD_BUFF = 0x44,
    WRITE_TX_FIFO = 0x66,
}

// Device states
#[allow(unused)]
enum State {
    NoChange = 0x00,
    Sleep = 0x01,
    SpiActive = 0x02,
    Ready = 0x03,
    Ready2 = 0x04,
    TxTune = 0x05,
    RxTune = 0x06,
    Tx = 0x07,
    Rx = 0x08,
}
