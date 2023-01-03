// SPDX-License-Identifier: GPL-3.0-or-later

use crate::system::System;
use super::Peripheral;

#[derive(Default)]
pub struct TIM {
    name: String,
    cr1: u16,
    arr: u16,
    psc: u16,
    rcr: u16,
    egr: u16,
    sr: u16,
    dier: u16,
    cnt: u16,
    ccr1: u16,
}

impl TIM {
    pub fn new(name: &str) -> Option<Box<dyn Peripheral>> {
        if name.starts_with("TIM") {
            let name = name.to_string();
            Some(Box::new(Self { name, ..TIM::default() }))
        } else {
            None
        }
    }
}

impl Peripheral for TIM {
    fn read(&mut self, _sys: &System, offset: u32) -> u32 {
        debug!("{} READ at offset=0x{:08x}", self.name, offset);
        match offset {
            0x0000 => {
                self.cr1 as u32
            }
            0x000c => {
                self.dier as u32
            }
            0x0024 => {
                self.cnt as u32
            }
            _ => {
                warn!("{} UNHANDLED READ!", self.name);
              0
            }
        }
    }

    fn write(&mut self, _sys: &System, offset: u32, value: u32) {
        match offset {
            0x0000 => {
                debug!("{} WRITE value=0x{:08x}", self.name, value);
                self.cr1 = value as u16;
                if self.cr1 & 1 == 1 {
                    debug!("--- {} COUNTER ENABLED ---", self.name)
                } else {
                    debug!("{} COUNTER DISABLED", self.name)
                }
            }
            0x002c => {
                debug!("{} WRITE value=0x{:08x}", self.name, value);
                self.arr = value as u16;
            }
            0x0028 => {
                debug!("{} WRITE PRESCALER value=0x{:08x}", self.name, value);
                self.psc = value as u16;
            }
            0x0030 => {
                debug!("{} WRITE value=0x{:08x}", self.name, value);
                self.rcr = value as u16;
            }
            0x0014 => {
                debug!("{} WRITE value=0x{:08x}", self.name, value);
                self.egr = value as u16;
                if self.egr & 1 == 1 {
                    debug!("{} GENERATE UPDATE EVENT", self.name);
                }
            }
            0x000C => {
                debug!("{} WRITE DIER value=0x{:08x}", self.name, value);
                self.dier = value as u16;
                if (self.dier >> 1) & 1 == 1 {
                    debug!("{} CC1 interrupt enabled!", self.name);
                }
            }
            0x0034 => {
                self.ccr1 = value as u16;
            }
            _ => {}
        }
    }
}
