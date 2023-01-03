// SPDX-License-Identifier: GPL-3.0-or-later

use std::time;
use crate::system::System;
use super::Peripheral;

#[derive(Default)]
pub struct RTC {
    name: String,
    tr: u32,  // RTC_TR: Time Register
    dr: u32,  // RTC_DR: Date Register
    cr: u32,  // RTC_CR: Control Register
    isr: u32, // RTC_ISR: Initialization and status register
    prer: u32, // RTC_PRER: RTC Prescaler register
    wutr: u32, // RTC_WUTR: Wakeup timer register
    calibr: u32, // RTC_CALIBR: Calibration register
    alrmar: u32, // RTC_ALRMAR: alarm A register
    alrmbr: u32, // RTC_ALRMAR: alarm B register
    wpr: u32, // RTC_WPR: Write protection register
    ssr: u32, // RTC_SSR: Sub second register
    shiftr: u32, // RTC_SHIFTR: Shift control register
    tstr: u32, // RTC_TSTR: time stamp register
    tsdr: u32, // RTC_TSDR: time stamp date register
    tsssr: u32, // RTC_TSTR: timestamp sub second register
    calr: u32, // RTC_CALR: calibration register
    tafcr: u32, // RTC_TAFCR: tamper and alternate function configuration register
    alrmassr: u32, // RTC_ALRMASSR: alarm A sub second register
    alrmbssr: u32, // RTC_ALRMBSSR: alarm B sub second register
    bkpxr: [u32; 20],
}

impl RTC {
    pub fn new(name: &str) -> Option<Box<dyn Peripheral>> {
        if name.starts_with("RTC") {
            let name = name.to_string();
            let isr = 0x0000_0007;
            let prer = 0x007F_00FF;
            let wutr =  0x0000_FFFF;
            let dr = 0x0000_2101;

            Some(Box::new(Self {
                name,
                dr,
                isr,
                prer,
                wutr,
                ..RTC::default()
            }))
        } else {
            None
        }
    }
}

impl Peripheral for RTC {
    fn read(&mut self, _sys: &System, offset: u32) -> u32 {
        match offset {
            0x00 => {
                debug!("RTC READ RTC_TR");
                self.tr
            }
            0x04 => {
                debug!("RTC READ RTC_DR");
                self.dr
            }
            0x08 => {
                debug!("RTC READ RTC_CR - Current value = {:032b}", self.cr);
                self.cr
            }
            0x0c => {
                // ALRAF:AlarmAflag
                // This flag is set by hardware when the time/date registers (RTC_TR and RTC_DR) match the Alarm A register (RTC_ALRMAR).
                //    This flag is cleared by software by writing 0.
                debug!("RTC READ RTC_ISR");
                // As this is all set by hardware, we always return one on read
                //1
                (1 << 5) | (1 << 6)
            }
            0x10 => {
                debug!("RTC READ RTC_PRER");
                self.prer
            }
            0x14 => {
                debug!("RTC READ RTC_WUTR");
                self.wutr
            }
            0x18 => {
                debug!("RTC READ RTC_CALIBR");
                self.calibr
            }
            0x1c => {
                debug!("RTC READ RTC_ALRMAR");
                self.alrmar
            }
            0x20 => {
                debug!("RTC READ RTC_ALRMBR");
                self.alrmbr
            }
            0x24 => {
                debug!("RTC READ RTC_WPR");
                self.wpr
            }
            0x28 => {
                debug!("RTC READ RTC_SSR");
                self.ssr
            }
            0x2c => {
                debug!("RTC READ RTC_SHIFTR");
                self.shiftr
            }
            0x30 => {
                debug!("RTC READ RTC_TSTR");
                self.tstr
            }
            0x34 => {
                debug!("RTC READ RTC_TSDR");
                self.tsdr
            }
            0x38 => {
                debug!("RTC READ RTC_TSSSR");
                self.tsssr
            }
            0x3c => {
                debug!("RTC READ RTC_CALR");
                self.calr
            }
            0x40 => {
                debug!("RTC READ RTC_TAFCR");
                self.tafcr
            }
            0x44 => {
                debug!("RTC READ RTC_ALRMASSR");
                self.alrmassr
            }
            0x48 => {
                debug!("RTC READ RTC_ALRMBSSR");
                self.alrmbssr
            }
            0x50 => {
                debug!("RTC READ RTC_BKPxR");
                self.tafcr
            }
            _ => {
                if offset >= 0x50 && offset <= 0x9c {
                    debug!("RTC READ RTC_BKPxR at offset={:08x}", offset);
                    0
                } else {
                    debug!("{} READ at offset=0x{:08x}", self.name, offset);
                    0
                }
            }
        }
    }

    fn write(&mut self, _sys: &System, offset: u32, value: u32) {
        match offset {
            0x00 => {
                debug!("RTC WRITE RTC_TR");
            }
            0x04 => {
                debug!("RTC WRITE RTC_DR");
            }
            0x08 => {
                debug!("RTC WRITE RTC_CR - Current value = {:032b}", self.cr);
                self.cr = value;
                debug!("RTC WRITE RTC_CR - New value = {:032b}", self.cr);
            }
            0x0c => {
                debug!("RTC WRITE RTC_ISR {:08x}", value);
            }
            0x10 => {
                debug!("RTC WRITE RTC_PRER {:08x}", value);
            }
            0x14 => {
                debug!("RTC WRITE RTC_WUTR {:08x}", value);
            }
            0x18 => {
                debug!("RTC WRITE RTC_CALIBR {:08x}", value);
            }
            0x1c => {
                debug!("RTC WRITE RTC_ALRMAR {:08x}", value);
            }
            0x20 => {
                debug!("RTC WRITE RTC_ALRMBR {:08x}", value);
            }
            0x24 => {
                debug!("RTC WRITE RTC_WPR {:08x}", value);
            }
            0x28 => {
                debug!("RTC WRITE RTC_SSR {:08x}", value);
            }
            0x2c => {
                debug!("RTC WRITE RTC_SHIFTR {:08x}", value);
            }
            0x30 => {
                debug!("RTC WRITE RTC_TSTR {:08x}", value);
            }
            0x34 => {
                debug!("RTC WRITE RTC_TSDR {:08x}", value);
            }
            0x38 => {
                debug!("RTC WRITE RTC_TSSSR {:08x}", value);
            }
            0x3c => {
                debug!("RTC WRITE RTC_CALR {:08x}", value);
            }
            0x40 => {
                debug!("RTC WRITE RTC_TAFCR {:08x}", value);
            }
            0x44 => {
                debug!("RTC WRITE RTC_ALRMASSR {:08x}", value);
            }
            0x48 => {
                debug!("RTC WRITE RTC_ALRMBSSR {:08x}", value);
            }
            0x50 => {
                debug!("RTC WRITE RTC_BKPxR {:08x}", value);
            }
            _ => {
                if offset >= 0x50 && offset <= 0x9c {
                    debug!("RTC WRITE RTC_BKPxR at offset={:08x}", offset);
                } else {
                    debug!("{} WRITE at offset=0x{:08x}", self.name, offset);
                }
            }
        }
    }
}
