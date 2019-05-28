use std::convert::TryInto;

use crate::mem::*;

type Instruction = u64;

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
