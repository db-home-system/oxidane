#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate embedded_hal;
extern crate panic_abort;
#[macro_use]
extern crate nb;
extern crate si4455;
extern crate stm32l151_hal as hal;

mod log;
mod radio_config;

use core::fmt::Write;

use hal::delay::Delay;
use hal::prelude::*;
use hal::rcc::{AHBPrescaler, APBPrescaler, PllDivider, PllMultiplier, PllSource, SystemClock};
use hal::serial::Serial;
use hal::spi::Spi;
use hal::stm32l151;
use log::Logger;
use rt::ExceptionFrame;
use si4455::Si4455;

entry!(main);

fn main() -> ! {
    let p = stm32l151::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    let clocks = rcc
        .cfgr
        .set_clock(
            SystemClock::PLL(PllSource::HSI, PllMultiplier::X6, PllDivider::Div3),
            AHBPrescaler::Div1,
            APBPrescaler::Div2,
            APBPrescaler::Div2,
        )
        .freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb);
    let mut gpioc = p.GPIOC.split(&mut rcc.ahb);

    /* Debug LED */
    let mut led = gpiob
        .pb4
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    /* Debug UART */
    let mut log = {
        let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
        let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

        let uart = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);
        let (tx, _) = uart.split();

        Logger::new(tx)
    };

    /* Si4455 */
    let mut si4455 = {
        /* SPI pins */
        let sck = gpioa
            .pa5
            .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr)
            .into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let miso = gpioa
            .pa6
            .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr)
            .into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let mosi = gpioa
            .pa7
            .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr)
            .into_af5(&mut gpioa.moder, &mut gpioa.afrl);

        /* Chip select */
        let mut nss = gpioa
            .pa4
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        /* Shutdown pin */
        let mut sdn = gpiob
            .pb10
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        /* Interrupt pin */
        let mut nirq = gpioc
            .pc13
            .into_pull_up_input(&mut gpioc.moder, &mut gpioc.pupdr);

        /* Vcc enable switch for RF */
        let mut ven_rf = gpiob
            .pb11
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        /* Power up the Si4455 */
        ven_rf.set_low();

        let mut spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            si4455::MODE,
            1.mhz(),
            clocks,
            &mut rcc.apb2,
        );

        Si4455::new(
            spi,
            nss,
            sdn,
            nirq,
            &mut delay,
            &radio_config::SI4455_CONFIG,
        )
    };

    if !si4455.begin(0, 17) {
        loop {
            led.toggle();
            delay.delay_ms(50_u16);
        }
    }

    loop {
        write!(&mut log, "Sending...").ok();
        write!(&mut log, "done!\n").ok();

        led.toggle();
        for _ in [0; 10].iter() {
            delay.delay_ms(100_u16);
        }
    }
}

exception!(*, default_handler);

fn default_handler(_irqn: i16) {}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}
