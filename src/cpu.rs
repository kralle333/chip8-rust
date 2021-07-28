use std::usize;

use crate::mem;

#[derive(Debug)]
pub enum Instruction {
    Invalid(u16),
    ClearScreen,
    Return,                  // 00EE
    Jump(u16),               // 1NNN
    Call(u16),               // 2NNN
    VxNNSkip(u16, u16),      // 3XNN
    VxNNNotSkip(u16, u16),   // 4XNN
    VxVySkip(u16, u16),      // 5XY0
    SetRegister(u16, u16),   // 6XNN
    AddRegister(u16, u16),   // 7XNN
    VxSetVy(u16, u16),       // 8XY0
    VxBitOrVy(u16, u16),     // 8XY1
    VxBitAndVy(u16, u16),    // 8XY2
    VxBitXOrVy(u16, u16),    // 8XY3
    VxBitAddVy(u16, u16),    // 8XY4
    VxSubVy(u16, u16),       // 8XY5
    VxBitShiftRVy(u16, u16), // 8XY6
    VxMinusVy(u16, u16),     // 8XY7
    VxBitShiftLVy(u16, u16), // 8XYE
    VxNotVySkip(u16, u16),   // 9XY0
    SetIndexRegister(u16),   // ANNN
    V0Jump(u16),             // BNNN
    VxRand(u16, u16),        // CXNN
    Draw(u8, u8, u16),       // DXYN
    KeyVxSkip(u16),          // EX9E
    KeyNotVxSkip(u16),       // EXA1
    SetTimerVx(u16),         // FX07
    GetKeyVx(u16),           // FX0A
    GetTimerVx(u16),         // FX15
    SetSoundTimerVx(u16),    // FX18
    AddIVx(u16),             // FX1E
    SetISprite(u16),         // FX29
    BCDVX(u16),              // FX33
    RegDump(u16),            // FX55
    RegLoad(u16),            // FX65
}


pub struct Cpu {
    pc: u16,
    sp: usize,
    stack: [u16; 64],
    index: u16,
    v: [u8; 16],
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            pc: 0x200,
            index: 0,
            sp: 0,
            stack: [0; 64],
            v: [0; 16],
        }
    }

    pub fn index(&self) -> u16 {
        self.index
    }

    fn unpack3(instr: u16) -> (u16, u16, u16) {
        let n1 = instr & 0x000F;
        let n2 = (instr & 0x00F0) >> 4;
        let n3 = (instr & 0x0F00) >> 8;
        (n3, n2, n1)
    }
    fn unpack2(instr: u16) -> (u16, u16) {
        let n = instr & 0x00FF;
        let n3 = (instr & 0x0F00) >> 8;
        //println!("unpack2: {:x} split into {:x} and {:x}",instr,n,n3);
        (n3, n)
    }

    pub fn inc_pc(&mut self) {
        self.pc += 2;
    }
    pub fn return_to_sp(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
        self.pc += 2;
    }

    pub fn set_index(&mut self, addr: u16) {
        self.index = addr;
    }

    pub fn set_v(&mut self, val: u8, index: usize) {
        self.v[index] = val
    }
    pub fn get_v(&self, index: usize) -> u8 {
        self.v[index]
    }

    pub fn jump(&mut self, addr: u16) {
        self.pc = addr
    }
    pub fn jump_store(&mut self, addr: u16) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = addr & 0x0FFF;
    }

    pub fn fetch_decode(&self, mem: &mem::Memory) -> Instruction {
        let opcode = mem.get_instruction(self.pc);
        //println!("0x{:x}",opcode);
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => Instruction::ClearScreen,
                    0x00EE => Instruction::Return,
                    _ => Instruction::Invalid(opcode), // machine code
                }
            }
            0x00EE => Instruction::Return,
            0x1000 => Instruction::Jump(opcode & 0x0FFF),
            0x2000 => Instruction::Call(opcode & 0x0FFF),
            0x3000 => {
                let (x, n) = Self::unpack2(opcode);
                Instruction::VxNNSkip(x, n)
            }
            0x4000 => {
                let (x, n) = Self::unpack2(opcode);
                Instruction::VxNNNotSkip(x, n)
            }
            0x5000 => {
                if opcode & 0x000F == 0 {
                    Instruction::VxVySkip((opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4)
                } else {
                    Instruction::Invalid(opcode)
                }
            }
            0x6000 => {
                let (x, n) = Self::unpack2(opcode);
                Instruction::SetRegister(x, n)
            }
            0x7000 => {
                let (x, nn) = Self::unpack2(opcode);
                Instruction::AddRegister(x, nn)
            }
            0x8000 => {
                let (x, y, _) = Self::unpack3(opcode);
                match opcode & 0x000F {
                    0x0 => Instruction::VxSetVy(x, y),
                    0x1 => Instruction::VxBitOrVy(x, y),
                    0x2 => Instruction::VxBitAndVy(x, y),
                    0x3 => Instruction::VxBitXOrVy(x, y),
                    0x4 => Instruction::VxBitAddVy(x, y),
                    0x5 => Instruction::VxSubVy(x, y),
                    0x6 => Instruction::VxBitShiftRVy(x, y),
                    0x7 => Instruction::VxMinusVy(x, y),
                    0xE => Instruction::VxBitShiftLVy(x, y),
                    _ => Instruction::Invalid(opcode)
                }
            }
            0x9000 => {
                if opcode & 0x000F == 0 {
                    let (x, y, _) = Self::unpack3(opcode);
                    Instruction::VxNotVySkip(x, y)
                } else {
                    Instruction::Invalid(opcode)
                }
            }
            0xA000 => Instruction::SetIndexRegister(opcode & 0x0FFF),
            0xB000 => Instruction::V0Jump((self.v[0] as u16) + opcode & 0x0FFF),
            0xC000 => {
                let (x, nn) = Self::unpack2(opcode);
                let random_factor: u16 = (rand::random::<f32>() * 256f32) as u16;
                Instruction::VxRand(x, nn & random_factor)
            }
            0xD000 => {
                let (x, y, n) = Self::unpack3(opcode);
                Instruction::Draw(self.v[x as usize], self.v[y as usize], n)
            }
            0xE000 => 
            {
                let x = (opcode & 0x0F00) >> 8;
                match opcode & 0x00FF {
                0x009E => Instruction::KeyVxSkip(x),
                0x00A1 => Instruction::KeyNotVxSkip(x),
                _ => (Instruction::Invalid(opcode))
                }
            },
            0xF000 => {
                let x = (opcode & 0x0F00) >> 8;
                match opcode & 0x00FF {
                    0x0007 => Instruction::SetTimerVx(x),
                    0x000A => Instruction::GetKeyVx(x),
                    0x0015 => Instruction::GetTimerVx(x),
                    0x0018 => Instruction::SetSoundTimerVx(x),
                    0x001E => Instruction::AddIVx(x),
                    0x0029 => Instruction::SetISprite(x),
                    0x0033 => Instruction::BCDVX(x),
                    0x0055 => Instruction::RegDump(x),
                    0x0065 => Instruction::RegLoad(x),
                    _ => Instruction::Invalid(opcode),
                }
            }
            _ => Instruction::Invalid(opcode), // machine code
        }
    }
}
