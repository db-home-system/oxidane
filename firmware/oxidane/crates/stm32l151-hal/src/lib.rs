//! HAL for the STM32L151 family of microcontrollers
//!
//! This is an implementation of the [`embedded-hal`] traits for the STM32L151 family of
//! microcontrollers.
//!
//! [`embedded-hal`]: https://github.com/japaric/embedded-hal
//!
//! # Requirements
//!
//! This crate requires `arm-none-eabi-gcc` to be installed and available in `$PATH` to build.
//!
//! # Usage
//!
//! To build applications (binary crates) using this crate follow the [cortex-m-quickstart]
//! instructions and add this crate as a dependency in step number 5 and make sure you enable the
//! "rt" Cargo feature of this crate.
//!
//! [cortex-m-quickstart]: https://docs.rs/cortex-m-quickstart/~0.3

#![deny(missing_docs)]
#![no_std]

extern crate cast;
extern crate cortex_m;
extern crate embedded_hal as hal;
extern crate nb;
extern crate stm32l1;
extern crate void;

pub use stm32l1::stm32l151;

pub mod flash;
pub mod gpio;
pub mod rcc;
pub mod time;
