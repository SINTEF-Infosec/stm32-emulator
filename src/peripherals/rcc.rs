// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::bail;
use crate::system::System;
use super::Peripheral;

pub struct Rcc {
    bdcr: u32,
}

impl Rcc {
    pub fn new(name: &str) -> Option<Box<dyn Peripheral>> {
        if name == "RCC" {
            Some(Box::new(Rcc{
                bdcr: 0x0,
            }))
        } else {
            None
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
            _ => 0
        }
    }

    fn write(&mut self, _sys: &System, _offset: u32, _value: u32) {
        match _offset {
            0x0070 => {
              self.bdcr = _value;
            }
            _ => {}
        }
    }
}
