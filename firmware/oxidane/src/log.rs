use core::fmt;

use hal::prelude::*;
use hal::serial;
use stm32l151::{USART1, USART2, USART3};

pub struct Logger<USART> {
    tx: serial::Tx<USART>,
}

impl<USART> Logger<USART> {
    pub fn new(tx: serial::Tx<USART>) -> Self {
        Logger { tx }
    }
}

macro_rules! logger {
    ($(
        $USARTX:ident,
    )+) => {
        $(
            impl fmt::Write for Logger<$USARTX> {
                fn write_str(&mut self, s: &str) -> fmt::Result {
                    let raw_s = s.as_bytes();
                    for &c in raw_s {
                        block!(self.tx.write(c)).map_err(|_| fmt::Error)?;
                    }
                    Ok(())
                }
            }
        )+
    }
}

logger! {
    USART1,
    USART2,
    USART3,
}
