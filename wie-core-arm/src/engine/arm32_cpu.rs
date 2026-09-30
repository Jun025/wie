use alloc::{boxed::Box, format, string::String, vec};
use core::sync::atomic::{AtomicBool, Ordering};

use arm32_cpu::{Cpu, Memory, Mode, reg};

use wie_util::{Result, WieError};

use crate::engine::{ArmEngine, ArmRegister, EngineRunResult, EngineStopReason, MemoryPermission};

pub struct Arm32CpuEngine {
    cpu: Cpu,
    mem: EmulatedMemory,
}

impl Arm32CpuEngine {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: EmulatedMemory::new(),
        }
    }

    fn is_svc_exception(&self) -> bool {
        self.cpu.reg_get(Mode::User, reg::PC) == 0x08 && (self.cpu.reg_get(Mode::User, reg::CPSR) & 0x1f) == 0x13
    }

    fn read_svc_result(&mut self) -> Result<EngineStopReason> {
        let lr = self.cpu.reg_get(Mode::Supervisor, reg::LR);
        let spsr = self.cpu.reg_get(Mode::Supervisor, reg::SPSR);

        let svc_address = lr.checked_sub(2).ok_or(WieError::InvalidMemoryAccess(lr))?;
        let mut svc_bytes = [0u8; 2];
        self.mem.read_range(svc_address, 2, &mut svc_bytes)?;
        let instruction = u16::from_le_bytes(svc_bytes);
        if instruction & 0xff00 != 0xdf00 {
            return Err(WieError::FatalError(format!(
                "Invalid Thumb SVC instruction {instruction:#06x} at {svc_address:#x}"
            )));
        }

        let category = instruction as u32 & 0xff;

        Ok(EngineStopReason::Svc { category, lr, spsr })
    }

    fn undefined_instruction_message(&self, pc: u32, cpsr: u32, lr: u32) -> String {
        let thumb = cpsr & (1 << 5) != 0;
        let mut bytes = [0u8; 4];
        let insn = match self.mem.read_range(pc, if thumb { 2 } else { 4 }, &mut bytes) {
            Ok(_) if thumb => format!("{:#06x}", u16::from_le_bytes([bytes[0], bytes[1]])),
            Ok(_) => format!("{:#010x}", u32::from_le_bytes(bytes)),
            Err(_) => "unreadable".into(),
        };

        format!(
            "Undefined instruction at pc={pc:#x} ({}) insn={insn} lr={lr:#x}",
            if thumb { "thumb" } else { "arm" }
        )
    }
}

impl ArmEngine for Arm32CpuEngine {
    fn run(&mut self, end: u32, count: u32) -> Result<EngineRunResult> {
        let mut instructions_executed = 0;
        let stop_reason = loop {
            let pc = self.cpu.reg_get(Mode::User, reg::PC);

            if self.is_svc_exception() {
                break self.read_svc_result()?;
            }

            if pc < 0x1000 {
                return Err(WieError::InvalidMemoryAccess(pc));
            }

            if pc == end {
                break EngineStopReason::End;
            }

            if instructions_executed == count {
                break EngineStopReason::Yield;
            }

            let cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            let lr = self.cpu.reg_get(Mode::User, reg::LR);
            let mut arm32cpu_memory = self.mem.as_arm32cpu_memory();

            if !(self.cpu.step(&mut arm32cpu_memory)) {
                // The caller's register dump is taken after every `run_function` on the way out has
                // restored its caller's context, so it shows the thread's initial registers (PC 0),
                // not these. This message is the only record of where the fault was.
                return Err(WieError::FatalError(self.undefined_instruction_message(pc, cpsr, lr)));
            }
            instructions_executed += 1;

            if let Some(x) = arm32cpu_memory.memory_error {
                return Err(WieError::InvalidMemoryAccess(x));
            }
        };

        Ok(EngineRunResult {
            stop_reason,
            instructions_executed,
        })
    }

    fn reg_write(&mut self, reg: ArmRegister, value: u32) {
        if reg == ArmRegister::PC && value % 2 == 1 {
            self.cpu.reg_set(Mode::User, reg.into_armv4t(), value - 1);

            let cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            self.cpu.reg_set(Mode::User, reg::CPSR, cpsr | (1 << 5)); // T bit

            return;
        }
        self.cpu.reg_set(Mode::User, reg.into_armv4t(), value);
    }

    fn reg_read(&self, reg: ArmRegister) -> u32 {
        self.cpu.reg_get(Mode::User, reg.into_armv4t())
    }

    fn mem_map(&mut self, address: u32, size: usize, _permission: MemoryPermission) {
        self.mem.map(address, size);
    }

    fn mem_write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        self.mem.write_range(address, data)
    }

    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> Result<usize> {
        self.mem.read_range(address, size, result)
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        self.mem.is_mapped(address, size)
    }
}

impl ArmRegister {
    fn into_armv4t(self) -> u8 {
        match self {
            ArmRegister::R0 => 0,
            ArmRegister::R1 => 1,
            ArmRegister::R2 => 2,
            ArmRegister::R3 => 3,
            ArmRegister::R4 => 4,
            ArmRegister::R5 => 5,
            ArmRegister::R6 => 6,
            ArmRegister::R7 => 7,
            ArmRegister::R8 => 8,
            ArmRegister::SB => 9,
            ArmRegister::SL => 10,
            ArmRegister::FP => 11,
            ArmRegister::IP => 12,
            ArmRegister::SP => reg::SP,
            ArmRegister::LR => reg::LR,
            ArmRegister::PC => reg::PC,
            ArmRegister::Cpsr => reg::CPSR,
        }
    }
}

const TOTAL_MEMORY: u64 = 0x100000000;
const PAGE_SIZE: usize = 0x10000;
const PAGE_MASK: u32 = (PAGE_SIZE - 1) as _;
const NULL_READ_LIMIT: u32 = 0x1000;
static ZERO_PAGE: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
// ponytail: one flag per process, not per engine — per-engine needs a field on EmulatedMemory.
static NULL_READ_SEEN: AtomicBool = AtomicBool::new(false);

struct EmulatedMemory {
    pages: Box<[Option<Box<[u8; PAGE_SIZE]>>]>,
}

impl EmulatedMemory {
    fn new() -> Self {
        Self {
            pages: vec![None; (TOTAL_MEMORY / PAGE_SIZE as u64) as usize].into_boxed_slice(),
        }
    }

    fn as_arm32cpu_memory(&mut self) -> Arm32CpuMemory<'_> {
        Arm32CpuMemory::new(self)
    }

    fn map(&mut self, address: u32, size: usize) {
        let page_start = address & !PAGE_MASK;
        let page_end = (address + size as u32 + PAGE_MASK) & !PAGE_MASK;

        for page in (page_start..page_end).step_by(PAGE_SIZE) {
            let page_data = &mut self.pages[page as usize / PAGE_SIZE];
            if page_data.is_none() {
                *page_data = Some(Box::new([0; PAGE_SIZE]));
            }
        }
    }

    fn read_range(&self, address: u32, size: usize, result: &mut [u8]) -> Result<usize> {
        let mut remaining_size = size;
        let mut current_address = address;

        while remaining_size > 0 {
            let page_address = current_address & !PAGE_MASK;
            let page_data = self.pages[page_address as usize / PAGE_SIZE]
                .as_ref()
                .ok_or(WieError::InvalidMemoryAccess(current_address))?;
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(remaining_size);

            result[size - remaining_size..size - remaining_size + available_bytes].copy_from_slice(&page_data[offset..offset + available_bytes]);
            remaining_size -= available_bytes;
            current_address += available_bytes as u32;
        }

        Ok(size)
    }

    fn write_range(&mut self, address: u32, data: &[u8]) -> Result<()> {
        let mut current_address = address;
        let mut data_index = 0;

        while data_index < data.len() {
            let page_address = current_address & !PAGE_MASK;
            let page_data = self.pages[page_address as usize / PAGE_SIZE]
                .as_mut()
                .ok_or(WieError::InvalidMemoryAccess(current_address))?;
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(data.len() - data_index);

            page_data[offset..offset + available_bytes].copy_from_slice(&data[data_index..data_index + available_bytes]);
            data_index += available_bytes;
            current_address += available_bytes as u32;
        }

        Ok(())
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        let page_start = address & !PAGE_MASK;
        let page_end = (address + size as u32 + PAGE_MASK) & !PAGE_MASK;

        if self.pages[page_start as usize / PAGE_SIZE].is_none() {
            return false;
        }

        for page in (page_start..page_end).step_by(PAGE_SIZE) {
            if self.pages[page as usize / PAGE_SIZE].is_none() {
                return false;
            }
        }

        true
    }
}

struct Arm32CpuMemory<'a> {
    emulated_memory: &'a mut EmulatedMemory,
    memory_error: Option<u32>,
}

impl<'a> Arm32CpuMemory<'a> {
    fn new(emulated_memory: &'a mut EmulatedMemory) -> Self {
        Self {
            emulated_memory,
            memory_error: None,
        }
    }

    /// A guest READ below [`NULL_READ_LIMIT`] of unmapped memory yields 0 instead of faulting;
    /// writes there, jumps there (`run`'s `pc < 0x1000`), and host-side reads still fault.
    ///
    /// Shipped titles read through null and go on: `2b1ed0c8d061` asks for a sound file whose name
    /// it misspells, keeps the empty sound, and later builds `*(null) + 8` as the data pointer it
    /// hands over with size 0. That typo is in the shipped binary, so the handset read address 0
    /// and survived; faulting here stopped the title at its first serve (`docs/report/0391`).
    fn read_page(&mut self, addr: u32) -> Option<&[u8; PAGE_SIZE]> {
        if addr < NULL_READ_LIMIT && self.emulated_memory.pages[0].is_none() {
            if !NULL_READ_SEEN.swap(true, Ordering::Relaxed) {
                tracing::warn!("guest read through a null pointer ({addr:#x}) — reads as 0; logged once per process");
            }
            return Some(&ZERO_PAGE);
        }
        self.get_page(addr).map(|x| &*x)
    }

    fn get_page(&mut self, addr: u32) -> Option<&mut [u8; PAGE_SIZE]> {
        let page_address = addr & !PAGE_MASK;
        let page_data = self.emulated_memory.pages[page_address as usize / PAGE_SIZE].as_mut();

        if let Some(x) = page_data {
            Some(x)
        } else {
            self.memory_error = Some(addr);
            None
        }
    }
}

impl Memory for Arm32CpuMemory<'_> {
    fn r8(&mut self, addr: u32) -> u8 {
        let offset = addr & PAGE_MASK;

        let page = self.read_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();

        data[offset as usize]
    }

    fn r16(&mut self, addr: u32) -> u16 {
        let offset = addr & PAGE_MASK;

        let page = self.read_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();

        u16::from_le_bytes(data[offset as usize..offset as usize + 2].try_into().unwrap())
    }

    fn r32(&mut self, addr: u32) -> u32 {
        let offset = addr & PAGE_MASK;

        let page = self.read_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();
        u32::from_le_bytes(data[offset as usize..offset as usize + 4].try_into().unwrap())
    }

    fn w8(&mut self, addr: u32, val: u8) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data[offset as usize] = val;
    }

    fn w16(&mut self, addr: u32, val: u16) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data[offset as usize..offset as usize + 2].copy_from_slice(&val.to_le_bytes());
    }

    fn w32(&mut self, addr: u32, val: u32) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data[offset as usize..offset as usize + 4].copy_from_slice(&val.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use core::mem::size_of;

    use arm32_cpu::Memory;

    use crate::engine::{ArmEngine, ArmRegister, EngineStopReason, MemoryPermission};

    use super::{Arm32CpuEngine, EmulatedMemory};

    #[test]
    fn run_reports_executed_instructions_at_budget_and_return_boundaries() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0xc0, 0x46, 0xc0, 0x46, 0x70, 0x47]).unwrap(); // nop; nop; bx lr
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.reg_write(ArmRegister::LR, 0x2000);

        for (budget, expected_count, at_end) in [(0, 0, false), (2, 2, false), (10, 1, true), (10, 0, true)] {
            let result = engine.run(0x2000, budget).unwrap();
            assert_eq!(result.instructions_executed, expected_count);
            assert!(matches!(
                (result.stop_reason, at_end),
                (EngineStopReason::End, true) | (EngineStopReason::Yield, false)
            ));
        }
    }

    #[test]
    fn undefined_instruction_names_the_faulting_pc_and_lr() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0xc0, 0x46, 0x00, 0xe8]).unwrap(); // nop; lone BLX suffix (undefined on ARMv4T)
        engine.reg_write(ArmRegister::Cpsr, 0x10);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.reg_write(ArmRegister::LR, 0x2345);

        let error = engine.run(0x2000, 10).err().unwrap();
        assert_eq!(
            alloc::format!("{error}"),
            "Fatal error: Undefined instruction at pc=0x1002 (thumb) insn=0xe800 lr=0x2345"
        );
    }

    #[test]
    fn page_table_is_heap_allocated() {
        assert_eq!(size_of::<EmulatedMemory>(), size_of::<Box<[Option<Box<[u8; super::PAGE_SIZE]>>]>>());
    }

    #[test]
    fn test_memory_basic() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x1000);
        memory.map(0x11000, 0x1000);
        memory.map(0x20000, 0x10000);

        memory.write_range(0x10000, &[123; 0x1000]).unwrap();

        let mut buf = [0; 0x1000];
        memory.read_range(0x10000, 0x1000, &mut buf).unwrap();
        assert_eq!(buf, [123; 0x1000]);

        memory.write_range(0x10900, &[100; 0x1000]).unwrap();

        memory.read_range(0x10900, 0x1000, &mut buf).unwrap();
        assert_eq!(buf, [100; 0x1000]);

        let mut arm32cpu_memory = memory.as_arm32cpu_memory();

        let r8 = arm32cpu_memory.r8(0x10000);
        assert_eq!(r8, 123);

        let r16 = arm32cpu_memory.r16(0x10000);
        assert_eq!(r16, 123 | (123 << 8));

        let r32 = arm32cpu_memory.r32(0x10000);
        assert_eq!(r32, 123 | (123 << 8) | (123 << 16) | (123 << 24));

        arm32cpu_memory.w8(0x10000, 12);
        let r8 = arm32cpu_memory.r8(0x10000);
        assert_eq!(r8, 12);

        arm32cpu_memory.w16(0x10000, 0x1234);
        let r16 = arm32cpu_memory.r16(0x10000);
        assert_eq!(r16, 0x1234);

        arm32cpu_memory.w32(0x10000, 0x12345678);
        let r32 = arm32cpu_memory.r32(0x10000);
        assert_eq!(r32, 0x12345678);
    }

    #[test]
    fn test_memory_unmapped_read() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x10000);

        let mut buf = [0; 0x1000];
        assert!(memory.read_range(0x1f500, 0x1000, &mut buf).is_err());

        let mut access = memory.as_arm32cpu_memory();
        assert_eq!(access.r32(0x20000), 0);
        assert_eq!(access.memory_error, Some(0x20000));
    }

    #[test]
    fn guest_reads_through_null_are_zero_but_writes_and_higher_reads_still_fault() {
        let mut memory = EmulatedMemory::new();
        let mut access = memory.as_arm32cpu_memory();

        assert_eq!((access.r8(0), access.r16(0x10), access.r32(0xffc)), (0, 0, 0));
        assert_eq!(access.memory_error, None);

        assert_eq!(access.r32(0x1000), 0);
        assert_eq!(access.memory_error, Some(0x1000));

        access.memory_error = None;
        access.w32(0x8, 1);
        assert_eq!(access.memory_error, Some(0x8));

        let mut buf = [0; 4];
        assert!(memory.read_range(0, 4, &mut buf).is_err(), "host-side reads keep faulting");
    }

    #[test]
    fn test_memory_unmapped_write() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x10000);

        assert!(memory.write_range(0x1f500, &[12; 0x1000]).is_err());

        let mut access = memory.as_arm32cpu_memory();
        access.w32(0x20000, 12);
        assert_eq!(access.memory_error, Some(0x20000));
    }
}
