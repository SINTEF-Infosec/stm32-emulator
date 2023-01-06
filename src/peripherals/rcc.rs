// SPDX-License-Identifier: GPL-3.0-or-later

use crate::system::System;
use super::Peripheral;

#[derive(Default)]
pub struct Rcc {
    apb1enr: u32,
    //RCC APB1 peripheral clock enable register
    cfgr: u32,
    // RCC clock configuration register
    // cr: u32, // RCC clock control register
    ppl_cfgr: u32,
    // RCC PLL configuration register
    ahb1enr: u32, // AHB1 peripheral clock enable register
    apb1rstr: u32, //RCC APB1 peripheral reset register
}

impl Rcc {
    pub fn new(name: &str) -> Option<Box<dyn Peripheral>> {
        if name == "RCC" {
            Some(Box::new(Rcc {
                ppl_cfgr: 0x2400_3010,
                ..Default::default()
                //cr: 0x0000_0081, // Reset 0x0000_XX81 where XX is undefined / we make everything ready
            }))
        } else {
            None
        }
    }
}


impl Peripheral for Rcc {
    fn read(&mut self, _sys: &System, offset: u32) -> u32 {
        match offset {
            0x0000 => 0x0f0f_ffff, // bypassed
            0x0008 => self.cfgr,
            0x0004 => self.ppl_cfgr,
            0x0020 => self.apb1rstr,
            0x0030 => self.ahb1enr,
            0x0040 => self.apb1enr,
            0x0074 => 0xff00_0003, // bypassed
            _ => {
                error!("NYI - {} READ at offset = {:08x}", "RCC", offset);
                std::process::exit(-1);
            }
        }
    }

    fn write(&mut self, _sys: &System, _offset: u32, _value: u32) {
        match _offset {
            0x0000 => {} // Ignored as we bypass
            0x0074 => {} // Ignored as we bypass
            0x0004 => self.ppl_cfgr = _value,
            0x0020 => self.apb1rstr = _value,
            0x0008 => {
                if _value & (1 << 1) != 0 {
                    self.cfgr |= 0x8 // select PLL as source for the clock and clear system clock switch
                } else {
                    self.cfgr = _value;
                }
            }
            0x0040 => self.apb1enr = _value,
            0x0030 => self.ahb1enr = _value,
            _ => {
                error!("NYI - {} WRITE at offset = {:08x} with value = {:08x}", "RCC", _offset, _value);
                std::process::exit(-1);
            }
        }
    }
}
