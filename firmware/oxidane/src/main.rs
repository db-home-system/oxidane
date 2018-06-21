#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate embedded_hal;
extern crate panic_abort;
extern crate stm32l151_hal as hal;

use cortex_m::asm;
use rt::ExceptionFrame;

use hal::prelude::*;
use hal::stm32l151;

entry!(main);

fn main() -> ! {
    let p = stm32l151::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb);

    let mut led = gpiob
        .pb4
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    led.set_high();

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
