//! Reset and Clock Control

use cast::u32;
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
                hclk: None,
                pclk1: None,
                pclk2: None,
                sysclk: None,
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

const HSI: u32 = 16_000_000; // Hz

const PLLMULS: [u32; 9] = [3, 4, 6, 8, 12, 16, 24, 32, 48];
const PLLDIVS: [u32; 3] = [2, 3, 4];

/// Clock configuration
pub struct CFGR {
    hclk: Option<u32>,
    pclk1: Option<u32>,
    pclk2: Option<u32>,
    sysclk: Option<u32>,
}

impl CFGR {
    /// Sets a frequency for the AHB bus
    pub fn hclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hclk = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB1 bus
    pub fn pclk1<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk1 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB2 bus
    pub fn pclk2<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk2 = Some(freq.into().0);
        self
    }

    /// Sets the system (core) frequency
    pub fn sysclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sysclk = Some(freq.into().0);
        self
    }

    /// Freezes the clock configuration, making it effective
    pub fn freeze(self, acr: &mut ACR) -> Clocks {
        let rcc = unsafe { &*RCC::ptr() };
        let pwr = unsafe { &*PWR::ptr() };

        // These defaults allow to achieve maximum clock frequency
        let sysclk = self.sysclk.unwrap_or(2 * HSI);
        let mut pllmul = 6;
        let mut plldiv = 3;

        // Find a multiplier and divider that match the target clock
        if HSI * pllmul / plldiv != sysclk {
            'outer: for &m in &PLLMULS {
                for &d in &PLLDIVS {
                    if HSI * m / d == sysclk {
                        pllmul = m;
                        plldiv = d;
                        break 'outer;
                    }
                }
            }
        }

        let pllmul_bits = PLLMULS
            .iter()
            .enumerate()
            .find(|(_, &val)| val == pllmul)
            .unwrap()
            .0 as u8;

        let plldiv_bits = plldiv as u8 - 1;

        let sysclk = HSI * pllmul / plldiv;
        assert!(sysclk <= 32_000_000);

        // AHB and core clock scaling
        let hpre_bits: u8 = match sysclk / self.hclk.unwrap_or(sysclk) {
            0 => unreachable!(),
            1 => 0b0111,
            2 => 0b1000,
            3...5 => 0b1001,
            6...11 => 0b1010,
            12...39 => 0b1011,
            40...95 => 0b1100,
            96...191 => 0b1101,
            192...383 => 0b1110,
            _ => 0b1111,
        };

        let hclk = sysclk / (1 << (hpre_bits - 0b0111));
        assert!(hclk <= 32_000_000);

        // APB1 clock scaling
        let ppre1_bits: u8 = match hclk / self.pclk1.unwrap_or(hclk) {
            0 => unreachable!(),
            1 => 0b011,
            2 => 0b100,
            3...5 => 0b101,
            6...11 => 0b110,
            _ => 0b111,
        };

        let ppre1 = 1 << (ppre1_bits - 0b011);
        let pclk1 = hclk / u32(ppre1);
        assert!(pclk1 <= 32_000_000);

        // APB2 clock scaling
        let ppre2_bits: u8 = match hclk / self.pclk2.unwrap_or(hclk) {
            0 => unreachable!(),
            1 => 0b011,
            2 => 0b100,
            3...5 => 0b101,
            6...11 => 0b110,
            _ => 0b111,
        };

        let ppre2 = 1 << (ppre2_bits - 0b011);
        let pclk2 = hclk / u32(ppre2);
        assert!(pclk2 <= 32_000_000);

        // Configure voltage range 1 (required to set clock to 32MHz)
        rcc.apb1enr.modify(|_, w| w.pwren().set_bit());
        pwr.cr.modify(|_, w| unsafe { w.vos().bits(0b01) });
        while !pwr.csr.read().vosf().bit_is_clear() {}

        // Configure FLASH before enabling PLL: 64-bit access, prefetch, 1 WS
        acr.acr().modify(|_, w| w.acc64().set_bit());
        acr.acr().modify(|_, w| w.prften().set_bit());
        acr.acr().modify(|_, w| w.latency().bit(hclk > 16_000_000));
        while acr.acr().read().latency().bit() != (hclk > 16_000_000) {}

        // Wait for HSI startup and trim it to factory values
        rcc.cr.modify(|_, w| w.hsion().set_bit());
        while !rcc.cr.read().hsirdy().bit_is_set() {}
        rcc.icscr.modify(|_, w| unsafe { w.hsitrim().bits(16) });

        // Configure PLL values
        rcc.cfgr.modify(|_, w| unsafe {
            w.pllmul()
                .bits(pllmul_bits)
                .plldiv()
                .bits(plldiv_bits)
                .pllsrc()
                .clear_bit()
        });

        // Wait for PLL startup
        rcc.cr.modify(|_, w| w.pllon().set_bit());
        while !rcc.cr.read().pllrdy().bit_is_set() {}

        // SW: PLL selected as system clock
        rcc.cfgr.modify(|_, w| unsafe {
            w.ppre2()
                .bits(ppre2_bits)
                .ppre1()
                .bits(ppre1_bits)
                .hpre()
                .bits(hpre_bits)
                .sw()
                .bits(0b11)
        });

        Clocks {
            hclk: Hertz(hclk),
            pclk1: Hertz(pclk1),
            pclk2: Hertz(pclk2),
            ppre1,
            ppre2,
            sysclk: Hertz(sysclk),
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
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
