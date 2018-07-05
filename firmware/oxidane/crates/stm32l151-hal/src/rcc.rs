//! Reset and Clock Control

use stm32l151::{rcc, PWR, RCC};

use flash::ACR;
use time::Hertz;

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb: AHB { _0: () },
            apb1: APB1 { _0: () },
            apb2: APB2 { _0: () },
            cfgr: CFGR {
                sysclk: SystemClock::MSI,
                hpre: AHBPrescaler::Div1,
                ppre1: APBPrescaler::Div1,
                ppre2: APBPrescaler::Div1,
            },
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    /// AMBA High-performance Bus (AHB) registers
    pub ahb: AHB,
    /// Advanced Peripheral Bus 1 (APB1) registers
    pub apb1: APB1,
    /// Advanced Peripheral Bus 2 (APB2) registers
    pub apb2: APB2,
    /// Clock configuration
    pub cfgr: CFGR,
}

/// AMBA High-performance Bus (AHB) registers
pub struct AHB {
    _0: (),
}

impl AHB {
    pub(crate) fn enr(&mut self) -> &rcc::AHBENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ahbenr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::AHBRSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ahbrstr }
    }
}

/// Advanced Peripheral Bus 1 (APB1) registers
pub struct APB1 {
    _0: (),
}

impl APB1 {
    pub(crate) fn enr(&mut self) -> &rcc::APB1ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb1enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::APB1RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb1rstr }
    }
}

/// Advanced Peripheral Bus 2 (APB2) registers
pub struct APB2 {
    _0: (),
}

impl APB2 {
    pub(crate) fn enr(&mut self) -> &rcc::APB2ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::APB2RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2rstr }
    }
}

const MSI_FREQ: u32 = 2_097_000; // Hz
const HSI_FREQ: u32 = 16_000_000; // Hz

const PLLMUL_VALUES: [u32; 9] = [3, 4, 6, 8, 12, 16, 24, 32, 48];
const HPRE_VALUES: [u32; 9] = [1, 2, 4, 8, 16, 64, 128, 256, 512];

/// AHB prescaler values
#[derive(Clone, Copy)]
pub enum AHBPrescaler {
    /// SYSCLK not divided
    Div1 = 0b0111,
    /// SYSCLK divided by 2
    Div2 = 0b1000,
    /// SYSCLK divided by 4
    Div4 = 0b1001,
    /// SYSCLK divided by 8
    Div8 = 0b1010,
    /// SYSCLK divided by 16
    Div16 = 0b1011,
    /// SYSCLK divided by 64
    Div64 = 0b1100,
    /// SYSCLK divided by 128
    Div128 = 0b1101,
    /// SYSCLK divided by 256
    Div256 = 0b1110,
    /// SYSCLK divided by 512
    Div512 = 0b1111,
}

/// APB prescaler values
#[derive(Clone, Copy)]
pub enum APBPrescaler {
    /// HCLK not divided
    Div1 = 0b011,
    /// HCLK divided by 2
    Div2 = 0b100,
    /// HCLK divided by 4
    Div4 = 0b101,
    /// HCLK divided by 8
    Div8 = 0b110,
    /// HCLK divided by 16
    Div16 = 0b111,
}

/// PLL multiplication factor
#[derive(Clone, Copy)]
pub enum PllMultiplier {
    /// PLLVCO = PLL clock entry x3
    X3 = 0b0000,
    /// PLLVCO = PLL clock entry x4
    X4 = 0b0001,
    /// PLLVCO = PLL clock entry x6
    X6 = 0b0010,
    /// PLLVCO = PLL clock entry x8
    X8 = 0b0011,
    /// PLLVCO = PLL clock entry x12
    X12 = 0b0100,
    /// PLLVCO = PLL clock entry x16
    X16 = 0b0101,
    /// PLLVCO = PLL clock entry x24
    X24 = 0b0110,
    /// PLLVCO = PLL clock entry x32
    X32 = 0b0111,
    /// PLLVCO = PLL clock entry x48
    X48 = 0b1000,
}

/// PLL output division
#[derive(Clone, Copy)]
pub enum PllDivider {
    /// PLL clock output = PLLVCO / 2
    Div2 = 0b01,
    /// PLL clock output = PLLVCO / 3
    Div3 = 0b10,
    /// PLL clock output = PLLVCO / 4
    Div4 = 0b11,
}

/// PLL entry clock source
#[derive(Clone, Copy)]
pub enum PllSource {
    /// HSI oscillator selected as PLL input clock
    HSI,
    /// HSE oscillator selected as PLL input clock
    HSE(Hertz),
}

/// System clock source
pub enum SystemClock {
    /// MSI oscillator
    MSI,
    /// HSI oscillator
    HSI,
    /// HSE oscillator
    HSE(Hertz),
    /// PLL output
    PLL(PllSource, PllMultiplier, PllDivider),
}

/// Clock configuration
pub struct CFGR {
    sysclk: SystemClock,
    hpre: AHBPrescaler,
    ppre1: APBPrescaler,
    ppre2: APBPrescaler,
}

impl CFGR {
    /// Configures the system clock tree
    pub fn set_clock(
        mut self,
        sysclk: SystemClock,
        hpre: AHBPrescaler,
        ppre1: APBPrescaler,
        ppre2: APBPrescaler,
    ) -> Self {
        self.sysclk = sysclk;
        self.hpre = hpre;
        self.ppre1 = ppre1;
        self.ppre2 = ppre2;
        self
    }

    /// Freezes the clock configuration, making it effective
    pub fn freeze(self, acr: &mut ACR) -> Clocks {
        let rcc = unsafe { &*RCC::ptr() };
        let pwr = unsafe { &*PWR::ptr() };

        // Configure voltage range 1 (required to set clock to 32MHz)
        rcc.apb1enr.modify(|_, w| w.pwren().set_bit());
        pwr.cr.modify(|_, w| unsafe { w.vos().bits(0b01) });
        while !pwr.csr.read().vosf().bit_is_clear() {}

        // Configure sysclk
        let sysclk = match self.sysclk {
            SystemClock::PLL(src, mul, div) => CFGR::setup_pll(src, mul, div),
            SystemClock::MSI => MSI_FREQ,
            _ => unimplemented!(),
        };
        assert!(sysclk <= 32_000_000);

        // Compute clock values based on selected source

        let hpre = HPRE_VALUES[self.hpre as usize - 0b0111];
        let hclk = sysclk / hpre;
        assert!(hclk <= 32_000_000);

        let ppre1 = 1 << (self.ppre1 as u8 - 0b011);
        let pclk1 = hclk / ppre1;
        assert!(pclk1 <= 32_000_000);

        let ppre2 = 1 << (self.ppre2 as u8 - 0b011);
        let pclk2 = hclk / ppre2;
        assert!(pclk2 <= 32_000_000);

        // Configure FLASH: 64-bit access, prefetch, 1 WS if HCLK > 16MHz
        acr.acr().modify(|_, w| w.acc64().set_bit());
        acr.acr().modify(|_, w| w.prften().set_bit());
        acr.acr().modify(|_, w| w.latency().bit(hclk > 16_000_000));
        while acr.acr().read().latency().bit() != (hclk > 16_000_000) {}

        // Perform clock switch
        rcc.cfgr.modify(|_, w| unsafe {
            w.hpre()
                .bits(self.hpre as u8)
                .ppre1()
                .bits(self.ppre1 as u8)
                .ppre2()
                .bits(self.ppre2 as u8)
                .sw()
                .bits(match self.sysclk {
                    SystemClock::MSI => 0b00,
                    SystemClock::HSI => 0b01,
                    SystemClock::HSE(_) => 0b10,
                    SystemClock::PLL(_, _, _) => 0b11,
                })
        });

        Clocks {
            hclk: Hertz(hclk),
            pclk1: Hertz(pclk1),
            pclk2: Hertz(pclk2),
            ppre1: ppre1 as u8,
            ppre2: ppre2 as u8,
            sysclk: Hertz(sysclk),
        }
    }

    /// Configure PLL clock output
    fn setup_pll(src: PllSource, mul: PllMultiplier, div: PllDivider) -> u32 {
        let rcc = unsafe { &*RCC::ptr() };

        let pllclk: u32 = match src {
            PllSource::HSI => {
                // Wait for HSI startup and trim it to factory values
                rcc.cr.modify(|_, w| w.hsion().set_bit());
                while !rcc.cr.read().hsirdy().bit_is_set() {}
                rcc.icscr.modify(|_, w| unsafe { w.hsitrim().bits(16) });

                HSI_FREQ
            }
            PllSource::HSE(freq) => {
                // Wait for HSE startup
                rcc.cr.modify(|_, w| w.hseon().set_bit());
                while !rcc.cr.read().hserdy().bit_is_set() {}

                freq.0
            }
        };

        // Configure PLL values
        rcc.cfgr.modify(|_, w| unsafe {
            w.pllmul()
                .bits(mul as u8)
                .plldiv()
                .bits(div as u8)
                .pllsrc()
                .bit(match src {
                    PllSource::HSI => false,
                    PllSource::HSE(_) => true,
                })
        });

        // Wait for PLL startup
        rcc.cr.modify(|_, w| w.pllon().set_bit());
        while !rcc.cr.read().pllrdy().bit_is_set() {}

        pllclk * PLLMUL_VALUES[mul as usize] / (div as u32 + 1)
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Debug, Clone, Copy)]
pub struct Clocks {
    hclk: Hertz,
    pclk1: Hertz,
    pclk2: Hertz,
    // TODO remove `allow`
    #[allow(dead_code)]
    ppre1: u8,
    // TODO remove `allow`
    #[allow(dead_code)]
    ppre2: u8,
    sysclk: Hertz,
}

impl Clocks {
    /// Returns the frequency of the AHB
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns the frequency of the APB1
    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    /// Returns the frequency of the APB2
    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    // TODO remove `allow`
    #[allow(dead_code)]
    pub(crate) fn ppre1(&self) -> u8 {
        self.ppre1
    }

    // TODO remove `allow`
    #[allow(dead_code)]
    pub(crate) fn ppre2(&self) -> u8 {
        self.ppre2
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
