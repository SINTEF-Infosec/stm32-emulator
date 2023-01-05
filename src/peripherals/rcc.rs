// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::bail;
use crate::peripherals::rcc::RccLsiRcOscillatorMode::{RccLsiRcOscillatorOff, RccLsiRcOscillatorOn};
use crate::system::System;
use super::Peripheral;

pub struct Rcc {
    bdcr: u32,
    csr: u32,
}

enum RccLsiRcOscillatorMode {
    RccLsiRcOscillatorOn,
    RccLsiRcOscillatorOff,
}

impl Rcc {
    pub fn new(name: &str) -> Option<Box<dyn Peripheral>> {
        if name == "RCC" {
            Some(Box::new(Rcc {
                bdcr: 0x0,
                csr: 0x0e00_0000,
            }))
        } else {
            None
        }
    }

    fn set_lsi_rc_oscillator(&mut self, mode: RccLsiRcOscillatorMode) {
        match mode {
            RccLsiRcOscillatorOn => {
                self.csr |= 2 // set the LSIRDY bit (Internal low-speed oscillator ready)
            }
            RccLsiRcOscillatorOff => {
                self.csr &= (0xffff_ffff ^ 2) // Clear LSIRDY bit
            }
        }
    }
}


impl Peripheral for Rcc {
    fn read(&mut self, _sys: &System, offset: u32) -> u32 {
        match offset {
            0x0000 => {
                // CR register
                // Return all the r to true. This is where the PLL ready flags are.
                //0b0010_0000_0010_0000_0000_0000_0010
                0xFFFF_FFFF
            }
            0x0008 => {
                // CFGR register
                0b1000
            }
            0x0070 => {
                // Backup Domain Control Register
                if self.bdcr != 0x0 {
                    self.bdcr | 2 // LSEON = 1
                } else {
                    self.bdcr
                }
            }
            0x0074 => {
                self.csr
            }
            _ => 0
        }
    }

    fn write(&mut self, _sys: &System, _offset: u32, _value: u32) {
        match _offset {
            0x0070 => {
                self.bdcr = _value;
            }
            0x0074 => {
                if ((_value << 31) >> 31) == 1 {
                    self.set_lsi_rc_oscillator(RccLsiRcOscillatorOn);
                } else if _value == 0 {
                    self.set_lsi_rc_oscillator(RccLsiRcOscillatorOff)
                }
            }
            _ => {}
        }

    }
}
