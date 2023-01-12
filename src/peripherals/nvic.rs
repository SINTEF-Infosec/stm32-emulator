// SPDX-License-Identifier: GPL-3.0-or-later

use std::process;
use std::sync::atomic::Ordering;

use unicorn_engine::{RegisterARM, Unicorn};
use crate::peripherals::nvic::irq::PENDSV;

use crate::system::System;
use super::Peripheral;

const BASE_OFFSET_INTERRUPT_PRIORITY_REGISTERS: u32 = 0x00000400;
const BASE_OFFSET_INTERRUPT_SET_ENABLE_REGISTERS: u32 = 0x00000100;


//#[derive(Default)]
pub struct Nvic {
    pub systick_period: Option<u32>,
    // ToDo: check if should be updated when touching syst_xxxx
    pub last_systick_trigger: u64,

    syst_rvr: u32,
    // SysTick Reload Value Register
    syst_cvr: u32,
    // SysTick Current Value Register
    syst_csr: u32,
    // SysTick Control and Status Register,
    vtor: u32,
    // Vector Table Offset Register
    cpacr: u32,
    // Coprocessor Access Control Register
    ccr: u32,
    // Configuration and Control Register
    shpr1: u32,
    // System Handler Priority Register 1
    shpr2: u32,
    // System Handler Priority Register 2
    shpr3: u32,
    // System Handler Priority Register 3
    shcsr: u32,
    // System Handler Control and State Register
    iprs: [u32; 124],
    // Interrupt Priority Registers
    iser: [u32; 16],
    // Interrupt Set-Enable Registers
    scr: u32,
    // System Control Register
    cfsr: u32,
    // Configurable Fault Status Register
    hfsr: u32,
    // HardFault Status Register
    icsr: u32,
    // Interrupt Control and State Register
    mmfar: u32,
    //MemManage Fault Address Register
    bfar: u32,
    // BusFault Address Register
    aircr: u32, // Application Interrupt and Reset Control Register

    mpu_type: u32, // MPU Type Register
    mpu_ctrl: u32,
    // MPU Control Register
    mpu_rnr: u32,
    // MPU Region Number Register
    mpu_rbar: u32,
    // MPU Region Base Address Register
    mpu_rasr: u32, // MPU Region Attribute and Size Register

    // 128 different interrupts. Good enough for now
    pending: u128,
    in_interrupt: bool,
}

const IRQ_OFFSET: i32 = 16;

pub mod irq {
    pub const SVCall: i32 = -5;
    pub const PENDSV: i32 = -2;
    pub const SYSTICK: i32 = -1;
}

// This is all poorly implemented. If this is not making much sense, it might be
// best to re-implement everything correctly. Right now, I'm just trying to get
// the saturn firmware to work just well enough.

impl Default for Nvic {
    fn default() -> Self {
        Self {
            systick_period: None,
            last_systick_trigger: 0,
            syst_rvr: 0x0000_0000,
            syst_cvr: 0x0000_0000,
            syst_csr: 0x0000_0000,
            icsr: 0x0000_0000,
            vtor: 0x0000_0000,
            cpacr: 0x22f2ffff, // All coprocessors available except CP15-CP12 and CP9-CP8
            iprs: [0x0000_0000; 124],
            iser: [0x0000_0000; 16],
            ccr: 0x0000_0200, // STKALIGN is recommend to reset to 1 by arm (p. 605 architecture doc).
            shpr1: 0x0000_0000,
            shpr2: 0x0000_0000,
            shpr3: 0x0000_0000,
            shcsr: 0x0000_0000,
            scr: 0x0000_0000,
            cfsr: 0x0000_0000,
            hfsr: 0x0000_0000,
            mmfar: 0x0000_0000,
            bfar: 0x0000_0000,
            aircr: 0x000_0000,
            mpu_type: 0x0000_0800,
            mpu_ctrl: 0x0000_0000,
            mpu_rnr: 0x0000_0000,
            mpu_rbar: 0x0000_0000,
            mpu_rasr: 0x0000_0000,
            pending: 0,
            in_interrupt: false,
        }
    }
}

impl Nvic {

    pub fn is_intr_pending(&mut self) -> bool {
        return self.pending != 0
    }

    fn systick_enabled(&mut self) -> bool {
        return self.syst_csr & 1 != 0;
    }

    fn systick_tickint(&mut self) -> bool {
        return self.syst_csr & 2 != 0;
    }

    pub fn tick(&mut self) {
        if !self.systick_enabled() {
            return;
        }

        if self.syst_cvr != 0 {
            self.syst_cvr -= 1;
            if self.syst_cvr == 0 {
                // if it reaches 0 from 1, we need to reload and perhaps have systick pending
                // and to set COUNTFLAG to 1
                self.syst_cvr = self.syst_rvr;
                if self.systick_tickint() {
                    self.set_intr_pending(irq::SYSTICK);
                }
                self.syst_csr |= (1 << 16);
            }
        } else {
            self.syst_cvr = self.syst_rvr;
        }
    }

    pub fn set_intr_pending(&mut self, irq: i32) {
        trace!("Set irq pending irq={}", irq);
        let bit = IRQ_OFFSET + irq;
        assert!(bit > 0);
        self.pending |= 1 << (IRQ_OFFSET + irq);
    }

    pub fn get_and_clear_next_intr_pending(&mut self) -> Option<i32> {
        if self.is_intr_pending() {
            let bit = self.pending.trailing_zeros();
            self.pending &= !(1 << bit);
            let irq = (bit as i32) - IRQ_OFFSET;
            Some(irq)
        } else {
            None
        }
    }

    pub fn maybe_set_systick_intr_pending(&mut self) {
        /*if let Some(systick_period) = self.systick_period {
            let n = crate::emulator::NUM_INSTRUCTIONS.load(Ordering::Relaxed);
            let delta_num_instructions = n - self.last_systick_trigger;
            if delta_num_instructions > (systick_period as u64) {
                self.last_systick_trigger = n;
                self.set_intr_pending(irq::SYSTICK);
            }
        }*/
    }

    fn are_interrupts_disabled(sys: &System) -> bool {
        let primask = sys.uc.borrow().reg_read(RegisterARM::PRIMASK).unwrap();
        //primask != 0;
        false
    }

    pub fn run_pending_interrupts(&mut self, sys: &System, vector_table_addr: u32) {
        //debug!("IN > run pending interrupts");

        //self.maybe_set_systick_intr_pending();

        if Self::are_interrupts_disabled(sys) || self.in_interrupt {
          //  debug!(" - interrupts disabled (is pending = {:?})", self.is_intr_pending());
            //debug!("OUT > run pending interrupts");
            return;
        }

        if let Some(irq) = self.get_and_clear_next_intr_pending() {
            //debug!("get and clear next intr pending = {:?}", irq);
            self.run_interrupt(sys, vector_table_addr, irq);
        }

        //debug!("OUT > run pending interrupts");
    }

    fn read_vector_addr(sys: &System, vector_table_addr: u32, irq: i32) -> u32 {
        // 4 because of ptr size
        let vaddr = vector_table_addr + 4 * (IRQ_OFFSET + irq) as u32;

        let mut vector = [0, 0, 0, 0];
        sys.uc.borrow().mem_read(vaddr as u64, &mut vector).unwrap();
        u32::from_le_bytes(vector)
    }

    // SPSEL, bit[1], 0 means we use MSP, 1 means we use PSP.
    // FPCA, bit[2], if the processor includes the FP extension.

    fn run_interrupt(&mut self, sys: &System, vector_table_addr: u32, irq: i32) {
        let vector = Self::read_vector_addr(sys, vector_table_addr, irq);

        let mut uc = sys.uc.borrow_mut();

        // SPSEL, bit[1], 0 means we use MSP, 1 means we use PSP.
        // FPCA, bit[2], if the processor includes the FP extension.
        let control_reg = uc.reg_read(RegisterARM::CONTROL).unwrap();
        let spsel = control_reg & (1 << 1) != 0;
        let fpca = control_reg & (2 << 1) != 0;

        trace!("Running interrupt irq={} spsel={} fpca={} vector={:#08x}",
            irq, spsel, fpca, vector);

        Self::push_regs(&mut uc, spsel, fpca);

        // LR meaning:
        //   EXC_RETURN    Return to      Return stack Frame type
        //   0xFFFF_FFE1   Handler mode   Main         Extended
        //   0xFFFF_FFE9   Thread mode    Main         Extended
        //   0xFFFF_FFED   Thread mode    Process      Extended
        //   0xFFFF_FFF1   Handler mode   Main         Basic
        //   0xFFFF_FFF9   Thread mode    Main         Basic
        //   0xFFFF_FFFD   Thread mode    Process      Basic

        // Right now, we don't supposed nested interrupts.
        let mut lr: u32 = 0xFFFF_FFE9;
        if spsel { lr |= 0b0000_0100; }
        if !fpca { lr |= 0b0001_0000; } // Yes, no fpca means the bit is set
        uc.reg_write(RegisterARM::LR, lr.into()).unwrap();

        uc.reg_write(RegisterARM::IPSR, irq as u64).unwrap();
        uc.reg_write(RegisterARM::PC, vector as u64).unwrap();

        self.in_interrupt = true;
    }

    pub fn return_from_interrupt(&mut self, sys: &System) {
        let mut uc = sys.uc.borrow_mut();

        let lr = uc.reg_read(RegisterARM::LR).unwrap();
        if lr & 0xFFFF_FF00 == 0xFFFF_FF00 {
            let spsel = lr & 0b0000_0100 != 0;
            let fpca = lr & 0b0001_0000 == 0; // 0 means yes here

            Self::pop_regs(&mut uc, spsel, fpca);

            trace!("Return from interrupt spsel={} fpca={} pc=0x{:08x}",
                spsel, fpca, uc.reg_read(RegisterARM::PC).unwrap());

            // SPSEL, bit[1], 0 means we use MSP, 1 means we use PSP.
            // FPCA, bit[2], if the processor includes the FP extension.
            let mut control_reg = 0;
            if spsel { control_reg |= 1 << 1; }
            if fpca { control_reg |= 2 << 1; }
            uc.reg_write(RegisterARM::CONTROL, control_reg).unwrap();
        } else {
            let control_reg = uc.reg_read(RegisterARM::CONTROL).unwrap();
            let spsel = control_reg & (1 << 1) != 0;
            let fpca = control_reg & (2 << 1) != 0;
            Self::pop_regs(&mut uc, spsel, fpca);

            trace!("Return from interrupt spsel={} fpca={} pc=0x{:08x} -- LR was not right",
                spsel, fpca, uc.reg_read(RegisterARM::PC).unwrap());
        }

        self.in_interrupt = false;
    }

    const CONTEXT_REGS_EXTENDED: [RegisterARM; 17] = [
        RegisterARM::FPSCR,
        RegisterARM::S15,
        RegisterARM::S14,
        RegisterARM::S13,
        RegisterARM::S12,
        RegisterARM::S11,
        RegisterARM::S10,
        RegisterARM::S9,
        RegisterARM::S8,
        RegisterARM::S7,
        RegisterARM::S6,
        RegisterARM::S5,
        RegisterARM::S4,
        RegisterARM::S3,
        RegisterARM::S2,
        RegisterARM::S1,
        RegisterARM::S0,
    ];

    const CONTEXT_REGS: [RegisterARM; 8] = [
        RegisterARM::XPSR,
        RegisterARM::PC,
        RegisterARM::LR,
        RegisterARM::R12,
        RegisterARM::R3,
        RegisterARM::R2,
        RegisterARM::R1,
        RegisterARM::R0,
    ];

    fn push_regs(uc: &mut Unicorn<()>, spsel: bool, fpca: bool) {
        let sp_reg = if spsel { RegisterARM::PSP } else { RegisterARM::MSP };
        let mut sp = uc.reg_read(sp_reg).unwrap();

        let mut push_reg = |reg| {
            let v = uc.reg_read(reg).unwrap() as u32;
            //trace!("push sp=0x{:08x} {:5?}=0x{:08x}", sp, reg, v);
            sp -= 4;
            uc.mem_write(sp, &v.to_le_bytes()).expect("Invalid SP pointer during interrupt");
        };

        if fpca {
            for reg in Self::CONTEXT_REGS_EXTENDED {
                push_reg(reg);
            }
        }
        for reg in Self::CONTEXT_REGS {
            push_reg(reg);
        }
        uc.reg_write(RegisterARM::SP, sp).unwrap();
    }

    fn pop_regs(uc: &mut Unicorn<()>, spsel: bool, fpca: bool) {
        let sp_reg = if spsel { RegisterARM::PSP } else { RegisterARM::MSP };
        let mut sp = uc.reg_read(sp_reg).unwrap();

        let mut pop_reg = |reg| {
            let mut v = [0, 0, 0, 0];
            uc.mem_read(sp, &mut v).expect("Invalid SP pointer during interrupt return");
            let v = u32::from_le_bytes(v);
            //trace!("pop sp=0x{:08x} {:5?}=0x{:08x}", sp, reg, v);
            sp += 4;
            uc.reg_write(reg, v as u64).unwrap();
        };

        for reg in Self::CONTEXT_REGS.iter().rev() {
            pop_reg(*reg);
        }
        if fpca {
            for reg in Self::CONTEXT_REGS_EXTENDED.iter().rev() {
                pop_reg(*reg);
            }
        }
        uc.reg_write(RegisterARM::SP, sp).unwrap();
    }
}

impl Peripheral for Nvic {
    fn read(&mut self, _sys: &System, _offset: u32) -> u32 {
        // Interrupt Priority Registers
        if _offset >= 0x400 && _offset < 0x5ec {
            let idx = (_offset - BASE_OFFSET_INTERRUPT_PRIORITY_REGISTERS) / 4;
            return self.iprs[idx as usize];
        }

        // Interrupt Set-Enable Registers
        if _offset > 0x100 && _offset < 0x13c {
            let idx = (_offset - BASE_OFFSET_INTERRUPT_SET_ENABLE_REGISTERS) / 4;
            return self.iser[idx as usize];
        }

        // Systick
        if _offset >= 0x010 && _offset < 0x0fc {
            return match _offset {
                0x010 => {
                    let ret = self.syst_csr;
                    if self.syst_csr & (1 << 16) != 0{
                        self.syst_csr ^= (1 << 16);
                    }
                    ret
                },
                0x014 => self.syst_rvr,
                0x018 => {
                    self.syst_cvr
                },
                _ => {
                    error!("NYI - {} READ at offset = {:08x}", "SYSTICK", _offset);
                    std::process::exit(-1);
                }
            };
        }

        return match _offset {
            0xd08 => self.vtor,
            0xd88 => self.cpacr,
            0xd04 => self.icsr,
            0xd14 => self.ccr,
            0xd20 => self.shpr3,
            0xd1c => self.shpr2,
            0xd18 => self.shpr1,
            0xd24 => self.shcsr,
            0xd10 => self.scr,
            0xd28 => self.cfsr,
            0xd2c => self.hfsr,
            0xd34 => self.mmfar,
            0xd38 => self.bfar,
            0xd90 => self.mpu_type,
            0xd94 => self.mpu_ctrl,
            0xd98 => self.mpu_rnr,
            0xd9c => self.mpu_rbar,
            0xda0 => self.mpu_rasr,
            _ => {
                error!("NYI - {} READ at offset = {:08x}", "NVIC", _offset);
                std::process::exit(-1);
            }
        };
    }

    fn write(&mut self, _sys: &System, _offset: u32, _value: u32) {
        // Interrupt Priority Registers
        if _offset >= 0x400 && _offset < 0x5ec {
            let idx = (_offset - BASE_OFFSET_INTERRUPT_PRIORITY_REGISTERS) / 4;
            self.iprs[idx as usize] = _value;
            return;
        }

        // Interrupt Set-Enable Registers
        if _offset >= 0x100 && _offset < 0x13c {
            let idx = (_offset - BASE_OFFSET_INTERRUPT_SET_ENABLE_REGISTERS) / 4;
            self.iser[idx as usize] = _value;
            return;
        }

        // Systick
        if _offset >= 0x010 && _offset < 0x0fc {
            return match _offset {
                0x010 => self.syst_csr = _value,
                0x014 => self.syst_rvr = _value,
                0x018 => self.syst_cvr = 0x0000_0000,
                _ => {
                    error!("NYI - {} WRITE at offset = {:08x} with value {:08x}", "SYSTICK", _offset, _value);
                    std::process::exit(-1);
                }
            };
        }

        match _offset {
            0xd04 => {
                if _value & (1 << 28) != 0 {
                    // Checking PENDSVSET (bit 28)
                    self.set_intr_pending(PENDSV);
                } else if _value & (1 << 27) != 0 {
                    // Handled when running pending interrupts
                } else {
                    error!("Interrupt Control and state register: NYI value = 0x{:08x}", _value);
                    process::exit(-1);
                }
            }
            0xd08 => self.vtor = _value,
            0xd88 => self.cpacr = _value,
            0xd14 => self.ccr = _value,
            0xd20 => self.shpr3 = _value,
            0xd1c => self.shpr2 = _value,
            0xd18 => self.shpr1 = _value,
            0xd24 => self.shcsr = _value,
            0xd10 => self.scr = _value,
            0xd28 => self.cfsr = _value,
            0xd2c => self.hfsr = _value,
            0xd94 => self.mpu_ctrl = _value,
            0xd98 => self.mpu_rnr = _value,
            0xd9c => self.mpu_rbar = _value,
            0xda0 => self.mpu_rasr = _value,
            0xd0c => self.aircr = _value,
            _ => {
                error!("NYI - {} WRITE at offset = {:08x} with value = {:08x}", "NVIC", _offset, _value);
                std::process::exit(-1);
            }
        }
    }
}

/// The next part is glue. Maybe we could have a better architecture.

pub struct NvicWrapper;

impl NvicWrapper {
    pub fn new(name: &str) -> Option<Box<dyn Peripheral>> {
        if name == "NVIC" {
            Some(Box::new(Self))
        } else {
            None
        }
    }
}

impl Peripheral for NvicWrapper {
    fn read(&mut self, sys: &System, offset: u32) -> u32 {
        sys.p.nvic.borrow_mut().read(sys, offset)
    }

    fn write(&mut self, sys: &System, offset: u32, value: u32) {
        sys.p.nvic.borrow_mut().write(sys, offset, value)
    }
}


/*
0xE000E100 B  REGISTER ISER0 (rw): Interrupt Set-Enable Register
0xE000E104 B  REGISTER ISER1 (rw): Interrupt Set-Enable Register
0xE000E108 B  REGISTER ISER2 (rw): Interrupt Set-Enable Register

0xE000E180 B  REGISTER ICER0 (rw): Interrupt Clear-Enable Register
0xE000E184 B  REGISTER ICER1 (rw): Interrupt Clear-Enable Register
0xE000E188 B  REGISTER ICER2 (rw): Interrupt Clear-Enable Register

0xE000E200 B  REGISTER ISPR0 (rw): Interrupt Set-Pending Register
0xE000E204 B  REGISTER ISPR1 (rw): Interrupt Set-Pending Register
0xE000E208 B  REGISTER ISPR2 (rw): Interrupt Set-Pending Register

0xE000E280 B  REGISTER ICPR0 (rw): Interrupt Clear-Pending Register
0xE000E284 B  REGISTER ICPR1 (rw): Interrupt Clear-Pending Register
0xE000E288 B  REGISTER ICPR2 (rw): Interrupt Clear-Pending Register

0xE000E300 B  REGISTER IABR0 (ro): Interrupt Active Bit Register
0xE000E304 B  REGISTER IABR1 (ro): Interrupt Active Bit Register
0xE000E308 B  REGISTER IABR2 (ro): Interrupt Active Bit Register

0xE000E400 B  REGISTER IPR0 (rw): Interrupt Priority Register
0xE000E404 B  REGISTER IPR1 (rw): Interrupt Priority Register
0xE000E408 B  REGISTER IPR2 (rw): Interrupt Priority Register
0xE000E40C B  REGISTER IPR3 (rw): Interrupt Priority Register
0xE000E410 B  REGISTER IPR4 (rw): Interrupt Priority Register
0xE000E414 B  REGISTER IPR5 (rw): Interrupt Priority Register
0xE000E418 B  REGISTER IPR6 (rw): Interrupt Priority Register
0xE000E41C B  REGISTER IPR7 (rw): Interrupt Priority Register
0xE000E420 B  REGISTER IPR8 (rw): Interrupt Priority Register
0xE000E424 B  REGISTER IPR9 (rw): Interrupt Priority Register
0xE000E428 B  REGISTER IPR10 (rw): Interrupt Priority Register
0xE000E42C B  REGISTER IPR11 (rw): Interrupt Priority Register
0xE000E430 B  REGISTER IPR12 (rw): Interrupt Priority Register
0xE000E434 B  REGISTER IPR13 (rw): Interrupt Priority Register
0xE000E438 B  REGISTER IPR14 (rw): Interrupt Priority Register
0xE000E43C B  REGISTER IPR15 (rw): Interrupt Priority Register
0xE000E440 B  REGISTER IPR16 (rw): Interrupt Priority Register
0xE000E444 B  REGISTER IPR17 (rw): Interrupt Priority Register
0xE000E448 B  REGISTER IPR18 (rw): Interrupt Priority Register
0xE000E44C B  REGISTER IPR19 (rw): Interrupt Priority Register
*/
