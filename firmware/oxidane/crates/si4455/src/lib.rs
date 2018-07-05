#![feature(asm)]
#![no_std]

extern crate embedded_hal as hal;
extern crate generic_array;
#[macro_use]
extern crate nb;

mod defs;
pub use defs::*;

use hal::blocking::delay::DelayMs;
use hal::blocking::spi;
use hal::digital::{InputPin, OutputPin};
use hal::spi::{FullDuplex, Mode, Phase, Polarity};

macro_rules! delay {
    () => {
        for _ in [0; 4_096].iter() {
            unsafe {
                asm!("NOP");
            }
        }
    };
}

#[derive(Debug)]
pub enum Error<E> {
    CommandError,
    TooManyBytes,
    EzCheckFailed(u8),
    InterruptError,
    Busy,
    ReadTimeout,
    Spi(E),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::Spi(error)
    }
}

const SI4455_CMD_ID_PART_INFO: u8 = 0x01;
const SI4455_CMD_ARG_COUNT_PART_INFO: usize = 1;
const SI4455_CMD_REPLY_COUNT_PART_INFO: usize = 9;

const SI4455_CMD_ID_FUNC_INFO: u8 = 0x10;
const SI4455_CMD_ARG_COUNT_FUNC_INFO: usize = 1;
const SI4455_CMD_REPLY_COUNT_FUNC_INFO: usize = 11;

const SI4455_CMD_ID_FIFO_INFO: u8 = 0x15;
const SI4455_CMD_ARG_COUNT_FIFO_INFO: usize = 2;
const SI4455_CMD_REPLY_COUNT_FIFO_INFO: usize = 2;

const SI4455_CMD_ID_FRR_A_READ: u8 = 0x50;

const SI4455_CMD_ID_GET_INT_STATUS: u8 = 0x20;
const SI4455_CMD_REPLY_COUNT_GET_INT_STATUS: usize = 8;

const SI4455_CMD_GET_INT_STATUS_REP_PACKET_SENT_PEND_BIT: u8 = 0x20;
const SI4455_CMD_GET_INT_STATUS_REP_PACKET_RX_PEND_BIT: u8 = 0x10;
const SI4455_CMD_GET_INT_STATUS_REP_CRC_ERROR_PEND_BIT: u8 = 0x08;
const SI4455_CMD_GET_INT_STATUS_REP_TX_FIFO_ALMOST_EMPTY_PEND_BIT: u8 = 0x02;
const SI4455_CMD_GET_INT_STATUS_REP_RX_FIFO_ALMOST_FULL_PEND_BIT: u8 = 0x01;
const SI4455_CMD_GET_INT_STATUS_REP_INVALID_SYNC_PEND_BIT: u8 = 0x20;
const SI4455_CMD_GET_INT_STATUS_REP_INVALID_PREAMBLE_PEND_BIT: u8 = 0x04;
const SI4455_CMD_GET_INT_STATUS_REP_FIFO_UNDERFLOW_OVERFLOW_ERROR_PEND_BIT: u8 = 0x20;
const SI4455_CMD_GET_INT_STATUS_REP_CMD_ERROR_PEND_BIT: u8 = 0x08;

const SI4455_CMD_ID_WRITE_TX_FIFO: u8 = 0x66;
const SI4455_CMD_ID_START_TX: u8 = 0x31;
const SI4455_CMD_ID_START_RX: u8 = 0x32;

const SI4455_CMD_ID_EZCONFIG_CHECK: u8 = 0x19;
const SI4455_CMD_GET_CHIP_STATUS_REP_CMD_ERROR_PEND_MASK: u8 = 0x08;

const SI4455_CONFIG: [u8; 273] = [
    0x07, 0x02, 0x01, 0x00, 0x01, 0x8C, 0xBA, 0x80, 0x05, 0x11, 0x01, 0x01, 0x00, 0x00, 0x08, 0x11,
    0x02, 0x04, 0x00, 0x08, 0x06, 0x04, 0x0A, 0x05, 0x11, 0x24, 0x01, 0x03, 0x64, 0x72, 0x66, 0xE2,
    0x48, 0x3A, 0xB0, 0xB0, 0x33, 0x00, 0xAA, 0x01, 0xFB, 0x3F, 0x85, 0x40, 0x2E, 0xB3, 0xE6, 0x03,
    0x58, 0xFE, 0x38, 0xA0, 0x91, 0x87, 0x6B, 0x34, 0x59, 0x41, 0xA4, 0xA6, 0xCC, 0x59, 0x54, 0xD7,
    0x01, 0x0F, 0x8E, 0xA0, 0xFA, 0x91, 0xB3, 0x3E, 0xAB, 0x55, 0x22, 0xF3, 0x84, 0xF5, 0x8E, 0x95,
    0x5D, 0x10, 0x3D, 0x8E, 0x1D, 0x18, 0x2F, 0x50, 0x56, 0xBA, 0x29, 0x9B, 0xE8, 0x16, 0x68, 0x5C,
    0x21, 0xB5, 0x28, 0x43, 0x70, 0xA8, 0x7F, 0x57, 0xD7, 0x26, 0x0E, 0xF3, 0xDC, 0xD9, 0xEE, 0xB9,
    0xF5, 0x85, 0xA4, 0x7B, 0xCA, 0x02, 0x13, 0x97, 0x32, 0x00, 0x43, 0x70, 0x6C, 0x84, 0x9A, 0xD1,
    0xBE, 0xE1, 0x71, 0xC1, 0xED, 0x1E, 0x7D, 0xA5, 0x23, 0x4B, 0xD8, 0x6B, 0x3A, 0xC3, 0x7D, 0x91,
    0x01, 0x00, 0x70, 0x66, 0x26, 0xF1, 0x58, 0xA3, 0x01, 0xB8, 0x7C, 0xBB, 0x64, 0xFB, 0x15, 0xE1,
    0x31, 0xD0, 0x8B, 0xBB, 0x10, 0xD7, 0x50, 0xEA, 0x7B, 0x43, 0xDF, 0x9C, 0xBD, 0x89, 0x35, 0x5A,
    0x4F, 0x45, 0x49, 0x5D, 0x09, 0xCD, 0x72, 0x2A, 0x9C, 0x9A, 0xAD, 0xB5, 0x9D, 0xD3, 0x87, 0x05,
    0x07, 0x27, 0x65, 0x48, 0xDA, 0xEE, 0xB9, 0x0C, 0xA2, 0xCC, 0xC2, 0xDF, 0x46, 0x7D, 0x3F, 0x78,
    0x96, 0x40, 0x57, 0x83, 0xFA, 0x31, 0x10, 0x3E, 0xEF, 0xC4, 0x57, 0x97, 0x93, 0x3C, 0x47, 0xEE,
    0xF2, 0x60, 0xC0, 0x37, 0x33, 0x3C, 0x87, 0x64, 0x42, 0xF9, 0x36, 0x5F, 0x71, 0xC9, 0x2E, 0xCE,
    0x14, 0xBE, 0x58, 0xC3, 0x51, 0x0D, 0x61, 0x99, 0x14, 0x0A, 0xE1, 0x84, 0x50, 0xDC, 0x8B, 0x05,
    0x23, 0xF5, 0x51, 0x03, 0x19, 0x50, 0x95, 0x08, 0x13, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00,
    0x00,
];

pub struct Si4455<SPI, NCS, SDN, NIRQ> {
    spi: SPI,
    ncs: NCS,
    sdn: SDN,
    nirq: NIRQ,

    channel_number: u8,
    packet_length: u16,

    cts_went_high: bool,
    system_error: bool,
    command_error: bool,
    crc_error_flag: bool,
    data_transmitted_flag: bool,
    data_available_flag: bool,
    tx_fifo_almost_empty_flag: bool,
    rx_fifo_almost_full_flag: bool,

    cmd_reply: [u8; 16],
}

impl<E, SPI, NCS, SDN, NIRQ> Si4455<SPI, NCS, SDN, NIRQ>
where
    SPI: spi::Write<u8, Error = E> + spi::Transfer<u8, Error = E> + FullDuplex<u8>,
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
    ) -> Si4455<SPI, NCS, SDN, NIRQ>
    where
        D: DelayMs<u8>,
    {
        Si4455 {
            spi,
            ncs,
            sdn,
            nirq,
            channel_number: 0,
            packet_length: 17,
            cts_went_high: false,
            system_error: false,
            command_error: false,
            crc_error_flag: false,
            data_transmitted_flag: false,
            data_available_flag: false,
            tx_fifo_almost_empty_flag: false,
            rx_fifo_almost_full_flag: false,
            cmd_reply: [0; 16],
        }
    }

    pub fn begin(&mut self, channel: u8, packet_length: u16) -> bool {
        self.channel_number = channel;
        self.system_error = false;

        if packet_length > 0 {
            self.packet_length = packet_length;
        }

        // Power Up the radio chip
        self.power_up();

        let mut retry_count = 10;
        // Load radio configuration
        while self.initialize(&SI4455_CONFIG) != 0 && retry_count > 0 {
            // Wait and retry
            self.power_up();
            retry_count -= 1;
        }

        if retry_count <= 0 {
            return false;
        }

        // Read ITs, clear pending ones
        self.read_interrupt_status(0, 0, 0);

        true
    }

    pub fn start_listening(&mut self, channel: u8, length: u16) {
        // Read ITs, clear pending ones
        self.read_interrupt_status(0, 0, 0);

        // Start Receiving packet on channel, START immediately, Packet n bytes long
        self.start_rx(channel, 0, length, 8, 8, 8);
    }

    fn power_up(&mut self) {
        self.reset();

        // Wait until reset timeout or Reset IT signal
        //for (unsigned int wDelay = 0; wDelay < RadioConfiguration.Radio_Delay_Cnt_After_Reset; wDelay++);
        for _ in [0; 10].iter() {
            delay!();
        }
    }

    fn reset(&mut self) {
        // Put radio in shutdown, wait then release
        self.assert_shutdown();
        for _ in [0; 10].iter() {
            delay!();
        }
        self.deassert_shutdown();
        for _ in [0; 10].iter() {
            delay!();
        }
        self.clear_cts();
    }

    pub fn initialize(&mut self, mut config_array: &[u8]) -> u8 {
        while config_array[0] != 0x00 {
            let cmd_bytes_count = config_array[0] as usize;
            config_array = &config_array[1..];

            if cmd_bytes_count > 16 {
                if config_array[0] == SI4455_CMD_ID_WRITE_TX_FIFO {
                    if cmd_bytes_count > 128 {
                        return 1;
                    }

                    config_array = &config_array[1..];
                    self.write_ez_config_array(&config_array[..cmd_bytes_count - 1]);

                    config_array = &config_array[cmd_bytes_count - 1..];
                    continue;
                } else {
                    return 1;
                }
            }

            let mut radio_cmd = [0; 16];

            radio_cmd
                .iter_mut()
                .zip(config_array[..cmd_bytes_count].iter())
                .for_each(|(d, s)| *d = *s);

            config_array = &config_array[cmd_bytes_count..];

            let mut response = [0];

            if self.send_command_and_get_response(&radio_cmd[..cmd_bytes_count], &mut response)
                != 0xFF
            {
                return 2;
            }

            if radio_cmd[0] == SI4455_CMD_ID_EZCONFIG_CHECK {
                if response[0] != 0 {
                    return 1;
                }
            }

            if self.irq_asserted() {
                // Get and clear all interrupts.  An error has occured...
                let it = self.read_interrupt_status(0, 0, 0);
                if (it[6] & SI4455_CMD_GET_CHIP_STATUS_REP_CMD_ERROR_PEND_MASK) != 0 {
                    return 1;
                }
            }
        }

        0
    }

    fn write_ez_config_array(&mut self, ez_config_array: &[u8]) {
        self.write_data(SI4455_CMD_ID_WRITE_TX_FIFO, ez_config_array, true);
    }

    pub fn start_tx(&mut self, channel: u8, condition: u8, length: u16) {
        let buffer = [
            SI4455_CMD_ID_START_TX,
            channel,
            condition,
            (length >> 8) as u8,
            length as u8,
            0,
        ];

        self.send_command(&buffer);
    }

    pub fn write_tx_fifo(&mut self, data: &[u8]) {
        self.write_data(SI4455_CMD_ID_WRITE_TX_FIFO, data, false);
    }

    fn start_rx(
        &mut self,
        channel: u8,
        condition: u8,
        length: u16,
        next_state_1: u8,
        next_state_2: u8,
        next_state_3: u8,
    ) {
        let buffer = [
            SI4455_CMD_ID_START_RX,
            channel,
            condition,
            (length >> 8) as u8,
            length as u8,
            next_state_1,
            next_state_2,
            next_state_3,
        ];

        self.send_command(&buffer);
    }

    fn read_interrupt_status(
        &mut self,
        clear_pending_ph: u8,
        clear_pending_modem: u8,
        clear_pending_chip: u8,
    ) -> &mut [u8] {
        let buffer = [
            SI4455_CMD_ID_GET_INT_STATUS,
            clear_pending_ph,
            clear_pending_modem,
            clear_pending_chip,
        ];

        let mut resp = [0; SI4455_CMD_REPLY_COUNT_GET_INT_STATUS];

        self.send_command_and_get_response(&buffer, &mut resp);

        self.cmd_reply
            .iter_mut()
            .zip(resp.iter())
            .for_each(|(d, s)| *d = *s);

        if self.system_error {
            // TODO: Invalid data returned! Clear it before?
            return &mut self.cmd_reply[..SI4455_CMD_REPLY_COUNT_GET_INT_STATUS];
        }

        self.process_ph_interrupt_pending(resp[2]);
        self.process_modem_interrupt_pending(resp[4]);
        self.process_chip_interrupt_pending(resp[6]);

        if self.command_error {
            self.command_error = false;

            let channel = self.channel_number;
            let length = self.packet_length;
            self.start_listening(channel, length);
        }

        if self.crc_error_flag {
            self.crc_error_flag = false;

            let channel = self.channel_number;
            let length = self.packet_length;
            self.start_listening(channel, length);
        }

        &mut self.cmd_reply[..SI4455_CMD_REPLY_COUNT_GET_INT_STATUS]
    }

    fn process_ph_interrupt_pending(&mut self, ph_pend: u8) -> bool {
        let mut clear_it = false;

        if (ph_pend & SI4455_CMD_GET_INT_STATUS_REP_PACKET_SENT_PEND_BIT) != 0 {
            self.data_transmitted_flag = true;
            clear_it = true;
        }

        if (ph_pend & SI4455_CMD_GET_INT_STATUS_REP_PACKET_RX_PEND_BIT) != 0 {
            // @todo Add circular buffer?
            self.data_available_flag = true;
            clear_it = true;
        }

        if (ph_pend & SI4455_CMD_GET_INT_STATUS_REP_CRC_ERROR_PEND_BIT) != 0 {
            self.crc_error_flag = true;
            self.reset_fifo();
            clear_it = true;
        }

        if (ph_pend & SI4455_CMD_GET_INT_STATUS_REP_TX_FIFO_ALMOST_EMPTY_PEND_BIT) != 0 {
            self.tx_fifo_almost_empty_flag = true;
            clear_it = true;
        }

        if (ph_pend & SI4455_CMD_GET_INT_STATUS_REP_RX_FIFO_ALMOST_FULL_PEND_BIT) != 0 {
            self.rx_fifo_almost_full_flag = true;
            clear_it = true;
        }

        clear_it
    }

    fn process_modem_interrupt_pending(&mut self, modem_pend: u8) -> bool {
        let mut clear_it = false;

        if (modem_pend & SI4455_CMD_GET_INT_STATUS_REP_INVALID_SYNC_PEND_BIT) != 0 {
            clear_it = true;
        }

        if (modem_pend & SI4455_CMD_GET_INT_STATUS_REP_INVALID_PREAMBLE_PEND_BIT) != 0 {
            clear_it = true;
        }

        clear_it
    }

    fn process_chip_interrupt_pending(&mut self, chip_pend: u8) -> bool {
        let mut clear_it = false;

        if (chip_pend & SI4455_CMD_GET_INT_STATUS_REP_FIFO_UNDERFLOW_OVERFLOW_ERROR_PEND_BIT) != 0 {
            self.reset_fifo();
            clear_it = true;
        }

        if (chip_pend & SI4455_CMD_GET_INT_STATUS_REP_CMD_ERROR_PEND_BIT) != 0 {
            self.reset_fifo();
            self.command_error = true;
            clear_it = true;
        }

        clear_it
    }

    fn set_system_error(&mut self) {
        if self.system_error {
            return;
        }

        self.system_error = true;
    }

    fn reset_fifo(&mut self) {
        let buffer = [SI4455_CMD_ID_FIFO_INFO, 0x03];

        self.send_command(&buffer);
    }

    pub fn read_part_info(&mut self) -> &mut [u8] {
        let buffer = [SI4455_CMD_ID_PART_INFO];
        let mut resp = [0; SI4455_CMD_REPLY_COUNT_PART_INFO];

        // TODO: check PART value, seems like MSB is null and it shouldn't be.
        self.send_command_and_get_response(&buffer, &mut resp);

        self.cmd_reply
            .iter_mut()
            .zip(resp.iter())
            .for_each(|(d, s)| *d = *s);

        &mut self.cmd_reply[..SI4455_CMD_REPLY_COUNT_PART_INFO]
    }

    pub fn read_func_info(&mut self) -> &mut [u8] {
        let buffer = [SI4455_CMD_ID_FUNC_INFO];
        let mut resp = [0; SI4455_CMD_REPLY_COUNT_FUNC_INFO];

        self.send_command_and_get_response(&buffer, &mut resp);

        self.cmd_reply
            .iter_mut()
            .zip(resp.iter())
            .for_each(|(d, s)| *d = *s);

        &mut self.cmd_reply[..SI4455_CMD_REPLY_COUNT_FUNC_INFO]
    }

    fn read_frr_a(&mut self) -> &mut [u8] {
        let mut resp = [0; 1];

        self.read_data(SI4455_CMD_ID_FRR_A_READ, &mut resp, false);

        self.cmd_reply
            .iter_mut()
            .zip(resp.iter())
            .for_each(|(d, s)| *d = *s);

        &mut self.cmd_reply[..1]
    }

    fn get_response(&mut self, data: &mut [u8]) -> u8 {
        let mut cts_val = 0_u8;
        let mut error_count = 1000;

        while error_count != 0 {
            self.clear_cs();
            self.spi_write_byte(0x44);
            cts_val = self.spi_read_byte();

            if cts_val == 0xFF {
                if data.len() > 0 {
                    self.spi_read_data(data);
                }
                self.set_cs();
                break;
            }

            self.set_cs();
            error_count -= 1;
            delay!();
        }

        if error_count == 0 {
            // ERROR! Should never take this long
            // @todo Error callback ?
            self.set_system_error();
            self.system_error = true; // Fix a strange "system error" bug...
            return 0;
        }

        if cts_val == 0xFF {
            self.cts_went_high = true;
        }

        self.system_error = false;

        cts_val
    }

    fn send_command(&mut self, data: &[u8]) {
        while !self.cts_went_high {
            self.poll_cts();
            if self.system_error {
                return;
            }
        }

        self.clear_cs();
        self.spi_write_data(data);
        self.set_cs();

        self.clear_cts();
    }

    fn send_command_and_get_response(&mut self, commandData: &[u8], responseData: &mut [u8]) -> u8 {
        self.send_command(commandData);
        self.get_response(responseData)
    }

    fn read_data(&mut self, command: u8, data: &mut [u8], poll_cts_flag: bool) {
        if poll_cts_flag {
            while !self.cts_went_high {
                self.poll_cts();
                if self.system_error {
                    return;
                }
            }
        }

        self.clear_cs();
        self.spi_write_byte(command);
        self.spi_read_data(data);
        self.set_cs();

        self.clear_cts();
    }

    fn write_data(&mut self, command: u8, data: &[u8], poll_cts_flag: bool) {
        if poll_cts_flag {
            while !self.cts_went_high {
                self.poll_cts();
                if self.system_error {
                    return;
                }
            }
        }

        self.clear_cs();
        self.spi_write_byte(command);
        self.spi_write_data(data);
        self.set_cs();

        self.clear_cts();
    }

    fn poll_cts(&mut self) -> u8 {
        self.get_response(&mut [])
    }

    fn clear_cts(&mut self) {
        self.cts_went_high = false;
    }

    fn assert_shutdown(&mut self) {
        self.sdn.set_high();
    }

    fn deassert_shutdown(&mut self) {
        self.sdn.set_low();
    }

    fn clear_cs(&mut self) {
        self.ncs.set_low();
    }

    fn set_cs(&mut self) {
        delay!();
        self.ncs.set_high();
    }

    fn irq_asserted(&mut self) -> bool {
        self.nirq.is_low()
    }

    fn spi_read_write_byte(&mut self, value: u8) -> u8 {
        let mut buf = [value];
        self.spi.transfer(&mut buf).ok();
        buf[0]
    }

    fn spi_write_byte(&mut self, value: u8) {
        self.spi_read_write_byte(value);
    }

    fn spi_read_byte(&mut self) -> u8 {
        self.spi_read_write_byte(0xFF)
    }

    fn spi_write_data(&mut self, data: &[u8]) {
        self.spi.write(data).ok();
    }

    fn spi_read_data(&mut self, data: &mut [u8]) {
        for b in data.iter_mut() {
            *b = 0xFF;
        }
        self.spi.transfer(data).ok();
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
