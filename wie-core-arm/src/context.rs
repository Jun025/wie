use core::clone::Clone;

#[derive(Clone)]
pub struct ArmCoreContext {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub sb: u32,
    pub sl: u32,
    pub fp: u32,
    pub ip: u32,
    pub sp: u32,
    pub lr: u32,
    pub pc: u32,
    pub cpsr: u32,
}

impl ArmCoreContext {
    pub fn words(&self) -> [u32; 17] {
        [
            self.r0, self.r1, self.r2, self.r3, self.r4, self.r5, self.r6, self.r7, self.r8, self.sb, self.sl, self.fp, self.ip, self.sp, self.lr,
            self.pc, self.cpsr,
        ]
    }
}
