use std::convert::TryInto;

type INSTRUCTION = u64;
const INSTRUCTION_SIZE: usize = std::mem::size_of::<INSTRUCTION>();

pub struct VM {
    memory: Vec<u8>,
    ip: u32,
    stackip: u32,
}

impl VM {
    fn current_instruction(&mut self, len: usize) -> INSTRUCTION {
        let ip = self.ip as usize;
        self.ip += 1;

        INSTRUCTION::from_le_bytes(
            self.memory
                .get(ip..(ip + INSTRUCTION_SIZE))
                .unwrap()
                .try_into()
                .unwrap(),
        )
    }

    fn push(&mut self, value: u8) {
        self.memory[self.stackip as usize] = value;
        self.stackip += 1;
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
