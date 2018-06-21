#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_abort;
extern crate stm32l1;

use cortex_m::asm;
use rt::ExceptionFrame;
use stm32l1::stm32l151;

entry!(main);

fn main() -> ! {
    let peripherals = stm32l151::Peripherals::take().unwrap();

    unsafe {
        peripherals
            .RCC
            .ahbenr
            .write(|w| w.bits(0b1 << 1));

        peripherals
            .GPIOB
            .moder
            .modify(|r, w| w.bits((r.bits() & !(0b11 << 8)) | (0b01 << 8)));

        peripherals.GPIOB.bsrr.write(|w| w.bits(0b1 << 4));
    }

    loop {
        asm::bkpt();
    }
}

exception!(*, default_handler);

fn default_handler(_irqn: i16) {}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}
