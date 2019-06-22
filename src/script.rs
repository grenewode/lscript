use std::convert::TryInto;

#[macro_use]
use crate::mem::*;

pub struct u4(u8);
impl u4 {
    pub fn new(value: u8) -> Self {
        match value {
            0..=0xf => u4(value),
            _ => panic!(""),
        }
    }
}

impl From<u64> for u4 {
    fn from(value: u64) -> u4 {
        Self::new(value as u8)
    }
}

pub struct u13(u16);

impl u13 {
    pub fn new(value: u16) -> Self {
        match value {
            0..=0x1fff => u13(value),
            _ => panic!(""),
        }
    }
}

impl From<u64> for u13 {
    fn from(value: u64) -> Self {
        Self::new(value as u16)
    }
}

pub enum TargetRegion {
    High,
    Low,
}

enum Input<C = u13> {
    Ref {
        register: u4,
        constant: C,
    },
    Value {
        target_region: TargetRegion,
        register: u4,
        constant: C,
    },
}

type LongInput = Input<u32>;

// Input = MEM(REG + CONST) | REG + CONST
// REF = MEM(REG + CONST) | REG_ID

// STORE REGISTER MEM
// LOAD MEM REGISTER
// LOAD 5 mem(%1+10) ==> MOVE %0+5 mem(%1+10)
// move mem(%1 + 10) %r

// COPY [SRC:VAL] [DEST:REF]
// ADD  [OP_A:VAL] [OP_B:VAL] [DEST:REF]
// JMP  [TEST:VAL] [LABEL:MEM_REF]

// [00] OPCODE 56
// COPY
// [OPCODE] [H/L] [LONG_INPUT] [REF]
//    8     1     1 + 4 + 32   1 + 4 + 13
// ADD
// [OPCODE] [INPUT]     [INPUT]    [REF]
//    8     1 + 4 + 13  1 + 4 + 13 1 + 4 + 13

// 0  COPY
// 1  JMP_IF_0
// 2  JMP_IF_NOT_0
// 3  JMP_IF_NEG
// 4  JMP_IF_POS
// 5  JMP
// 6  CMP
// 7  SUB
// 8  ADD
// 9  MUL
// 10 DIV
// 11 SHIFT_RIGHT
// 12 SHIFT_LEFT
// 13 BIT_OR
// 14 BIT_AND
// 15 BIT_NOT
// 16 BIT_XOR
// 17 F_ADD
// 18 F_SUB
// 19 F_MUL
// 20 F_DIV
// 21 CONVERT_F_I
// 22 CONVERT_F_U
// 23

/// [00][00 00][]
pub struct Instruction(u64);

impl Instruction {
    pub fn read_opcode(&mut self) -> u8 {
        let opcode = (self.0 & 0xff) as u8;
        self.0 >>= 8;
        opcode
    }

    fn read_input<C>(&mut self, const_bits: u8) -> Input<C>
    where
        C: From<u64>,
    {
        let const_mask = (1u64 << const_bits) - 1;

        let is_mem = (self.0 & 0x1) != 0;
        self.0 >>= 1;
        let register = u4::from(self.0 & 0xf);
        self.0 >>= 4;
        let constant = C::from(self.0 & const_mask);

        if is_mem {
            Input::Ref {}
        }
    }
}

impl Storable for Instruction {
    fn store_to(&self, dest: &mut Mem, start: PtrType) -> Result<PtrType, MemError> {
        self.0.store_to(dest, start)
    }
}

impl Loadable for Instruction {
    fn load_from(src: &Mem, start: PtrType) -> Result<(PtrType, Self), MemError> {
        let (end, value) = src.load(start)?;
        Ok((end, Instruction(value)))
    }
}

pub struct VM {
    memory: MemBuf,
    ip_ptr: u32,
    stack_ptr: u32,
}

impl VM {
    fn pop_instruction(&mut self, len: usize) -> Instruction {
        self.memory.load_and_advance(&mut self.ip_ptr).unwrap()
    }

    fn pop<T>(&mut self) -> T
    where
        T: Loadable,
    {
        self.memory.load_and_advance(&mut self.stack_ptr).unwrap()
    }

    fn push<T>(&mut self, value: &T)
    where
        T: Storable,
    {
        self.memory
            .store_and_advance(&mut self.stack_ptr, value)
            .unwrap()
    }

    pub fn exec(&mut self) {
        // let instruction = self.memory.get(self.ip).unwrap();
        // self.ip += 1;

        // match instruction {
        //     0x0 => { /* NOP */ }
        //     0x1 => {}
        // }
    }
}
