use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fs;
use std::num::Wrapping;
use std::usize;

use crate::cpu;
use crate::mem;
use crate::video;

const ROMS_DIR : &str = "<YOUR ROM PATH HERE>";

pub struct System {
    video: video::Video,
    cpu: cpu::Cpu,
    mem: mem::Memory,
    should_draw: bool,
    should_play_sound: bool,
    key_map: [sdl2::keyboard::Keycode; 16],
    delay_timer: u8,
    sound_timer: u8,
}
impl System {
    pub fn new() -> System {
        let new_system = System {
            video: video::Video::new(),
            cpu: cpu::Cpu::new(),
            mem: mem::Memory::new(),
            should_draw: false,
            should_play_sound: false,
            key_map: [
                Keycode::X,    // 0x0
                Keycode::Num1, // 0x1
                Keycode::Num2, // 0x2
                Keycode::Num3, // 0x3
                Keycode::Q,    // 0x4
                Keycode::W,    // 0x5
                Keycode::E,    // 0x6
                Keycode::A,    // 0x7
                Keycode::S,    // 0x8
                Keycode::D,    // 0x9
                Keycode::Z,    // 0xA
                Keycode::C,    // 0xB
                Keycode::Num4, // 0xC
                Keycode::R,    // 0xD
                Keycode::F,    // 0xE
                Keycode::V,    // 0xF
            ],
            delay_timer: 0,
            sound_timer: 0,
        };
        new_system
    }

    pub fn load_game(&mut self, file_path: &str) {
        let file_path = ROMS_DIR.to_owned() + file_path;
        match fs::read(file_path) {
            Ok(data) => {
                self.mem.load(data);
            }
            Err(e) => {
                panic!("ERROR {}", e);
            }
        }
    }

    pub fn should_draw(&self) -> bool {
        self.should_draw
    }
    pub fn should_play_sound(&self) -> bool {
        self.should_play_sound
    }


    pub fn tick(&mut self, keys_pressed: &HashMap<sdl2::keyboard::Keycode, i32>) {
        let instr = self.cpu.fetch_decode(&self.mem);

        match instr {
            cpu::Instruction::Invalid(x) => println!("Unknown opcode 0x{:x}", x),
            _ => println!("{:?}", instr),
        }

        match instr {
            cpu::Instruction::Invalid(_) => self.cpu.inc_pc(),
            cpu::Instruction::ClearScreen => {
                self.cpu.inc_pc();
                self.video.clear();
                self.should_draw = true;
            }
            cpu::Instruction::Return => self.cpu.return_to_sp(),
            cpu::Instruction::Jump(addr) => self.cpu.jump(addr),
            cpu::Instruction::Call(addr) => self.cpu.jump_store(addr),
            cpu::Instruction::Draw(x, y, n) => {
                self.cpu.inc_pc();
                self.should_draw = true;
                match self.video.draw_sprite(x, y, n, self.cpu.index(), &self.mem) {
                    true => self.cpu.set_v(1, 0xF),
                    false => self.cpu.set_v(0, 0xF),
                }
            }
            cpu::Instruction::VxNNSkip(x, nn) => {
                self.cpu.inc_pc();
                if self.cpu.get_v(x as usize) == nn as u8 {
                    self.cpu.inc_pc();
                }
            }
            cpu::Instruction::VxNNNotSkip(x, nn) => {
                self.cpu.inc_pc();
                if self.cpu.get_v(x as usize) != nn as u8 {
                    self.cpu.inc_pc();
                }
            }
            cpu::Instruction::VxVySkip(x, y) => {
                self.cpu.inc_pc();
                if self.cpu.get_v(x as usize) == self.cpu.get_v(y as usize) {
                    self.cpu.inc_pc();
                }
            }
            cpu::Instruction::SetRegister(x, nn) => {
                self.cpu.inc_pc();
                //println!("Setting register: 0x{:x} to value 0x{:x}", x, nn);
                self.cpu.set_v(nn as u8, x as usize);
            }
            cpu::Instruction::AddRegister(x, nn) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let new_val = Wrapping(self.cpu.get_v(x)) + Wrapping(nn as u8);
                self.cpu.set_v(new_val.0, x);
            }
            cpu::Instruction::VxSetVy(x, y) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let y = y as usize;
                let y_val = self.cpu.get_v(y);
                self.cpu.set_v(y_val, x);
            }
            cpu::Instruction::VxBitOrVy(x, y) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                let val_y = self.cpu.get_v(y as usize);
                self.cpu.set_v(val_x | val_y, x);
            }
            cpu::Instruction::VxBitAndVy(x, y) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                let val_y = self.cpu.get_v(y as usize);
                self.cpu.set_v(val_x & val_y, x);
            }
            cpu::Instruction::VxBitXOrVy(x, y) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                let val_y = self.cpu.get_v(y as usize);
                self.cpu.set_v(val_x ^ val_y, x);
            }
            cpu::Instruction::VxBitAddVy(x, y) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                let val_y = self.cpu.get_v(y as usize);
                if val_y > (0xFF - val_x) {
                    self.cpu.set_v(1, 0xF); // Carry
                } else {
                    self.cpu.set_v(0, 0xF);
                }
                let result = Wrapping(val_x) + Wrapping(val_y);
                println!("{}+{}={}",val_x,val_y,result);
                self.cpu.set_v(result.0, x);
            }
            cpu::Instruction::VxSubVy(x, y) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                let val_y = self.cpu.get_v(y as usize);
                if val_y > val_x {
                    self.cpu.set_v(0, 0xF); //Borrow
                } else {
                    self.cpu.set_v(1, 0xF);
                }
                self.cpu.set_v((Wrapping(val_x) - Wrapping(val_y)).0, x);
            }
            cpu::Instruction::VxBitShiftRVy(x, _) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                self.cpu.set_v(val_x & 0b0001, 0xF);
                //println!("{:x}>>1={:x}",val_x,val_x>>1);
                self.cpu.set_v(val_x >> 1, x);
            }
            cpu::Instruction::VxBitShiftLVy(x, _) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                self.cpu.set_v(val_x & 0b1000, 0xF);
                self.cpu.set_v(val_x << 1, x);
            }
            cpu::Instruction::VxMinusVy(x, y) => {
                self.cpu.inc_pc();
                let x = x as usize;
                let val_x = self.cpu.get_v(x as usize);
                let val_y = self.cpu.get_v(y as usize);
                if val_x > val_y {
                    self.cpu.set_v(0, 0xF); //Borrow
                } else {
                    self.cpu.set_v(1, 0xF);
                }
                self.cpu.set_v(val_y - val_x, x);
            }
            cpu::Instruction::VxNotVySkip(x, y) => {
                self.cpu.inc_pc();
                let val_x = self.cpu.get_v(x as usize);
                let val_y = self.cpu.get_v(y as usize);
                if val_x != val_y {
                    self.cpu.inc_pc()
                }
            }
            cpu::Instruction::SetIndexRegister(nnn) => {
                self.cpu.inc_pc();
                self.cpu.set_index(nnn);
            }
            cpu::Instruction::V0Jump(addr) => self.cpu.jump(addr),
            cpu::Instruction::VxRand(x, n) => {
                self.cpu.inc_pc();
                self.cpu.set_v(n as u8, x as usize);
            }
            cpu::Instruction::KeyVxSkip(x) => {
                self.cpu.inc_pc();
                let vx_key = self.cpu.get_v(x as usize);
                if vx_key <= 0xF {
                    let requested_key = &self.key_map[vx_key as usize];
                    match keys_pressed.get(requested_key) {
                        Some(k) => {
                            if *k == 1 {
                                self.cpu.inc_pc();
                            }
                        }
                        None => {
                            println!("Unknown key {}", requested_key)
                        }
                    }
                }
            }
            cpu::Instruction::KeyNotVxSkip(x) => {
                self.cpu.inc_pc();
                let vx_key = self.cpu.get_v(x as usize);
                if vx_key <= 0xF {
                    let requested_key = &self.key_map[vx_key as usize];
                    match keys_pressed.get(requested_key) {
                        Some(k) => {
                            if *k == 0 {
                                self.cpu.inc_pc();
                            }
                        }
                        None => {
                            println!("Unknown key {}", requested_key)
                        }
                    }
                }
            }
            cpu::Instruction::GetKeyVx(x) => {
                let mut press_detected = false;
                for i in 0..16 {
                    let requested_key = &self.key_map[i];
                    match keys_pressed.get(requested_key) {
                        Some(k) => {
                            if *k == 1 {
                                press_detected = true;
                                self.cpu.set_v(i as u8, x as usize);
                            }
                        }
                        None => {
                            println!("Unknown key {}", requested_key)
                        }
                    }
                }
                if press_detected {
                    self.cpu.inc_pc();
                }
            }
            cpu::Instruction::SetTimerVx(x) => {
                self.cpu.set_v(self.delay_timer, x as usize);
                self.cpu.inc_pc()
            }
            cpu::Instruction::GetTimerVx(x) => {
                self.delay_timer = self.cpu.get_v(x as usize);
                self.cpu.inc_pc()
            }
            cpu::Instruction::SetSoundTimerVx(x) => {
                self.sound_timer = self.cpu.get_v(x as usize);
                self.cpu.inc_pc()
            }
            cpu::Instruction::AddIVx(x) => {
                let vx = self.cpu.get_v(x as usize) as u16;
                if vx + self.cpu.index() > 0xFFF{
                    self.cpu.set_v(1,0xF)
                }else{
                    self.cpu.set_v(0,0xF)
                }
                let new_value =  vx + self.cpu.index();
                self.cpu.set_index(new_value);
                self.cpu.inc_pc();
            }
            cpu::Instruction::SetISprite(x) => {
                let addr = (self.cpu.get_v(x as usize) * 0x5) as u16;
                self.cpu.set_index(addr);
                self.cpu.inc_pc()
            }
            cpu::Instruction::BCDVX(x) => {
                let i = self.cpu.index() as usize;
                let vx = self.cpu.get_v(x as usize);
                self.mem.set(i, vx / 100);
                self.mem.set(i + 1, (vx / 10) % 10);
                self.mem.set(i + 2, (vx % 100) % 10);
                //println!("vx: {} i: {} i+1: {} i+2: {}",vx,vx / 100,(vx / 10) % 10, (vx % 100) % 10);
                self.cpu.inc_pc();
            }
            cpu::Instruction::RegDump(x) => {
                for vx in 0..x + 1 {
                    let i = self.cpu.index() + vx;
                    self.mem.set(i as usize, self.cpu.get_v(vx as usize));
                }
                self.cpu.set_index(x + self.cpu.index() + 1);
                self.cpu.inc_pc()
            }

            cpu::Instruction::RegLoad(x) => {
                for vx in 0..x + 1 {
                    let i = self.cpu.index() + vx;
                    self.cpu
                        .set_v(self.mem.get_byte(i as u16) as u8, vx as usize);
                }
                self.cpu.set_index(x + self.cpu.index() + 1);
                self.cpu.inc_pc()
            }
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        self.should_play_sound = self.sound_timer == 1;
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>) {
        self.video.draw(canvas);
        self.should_draw = false;
    }

    // for tests

    pub fn load_test(&mut self, data:Vec<u8>){
        self.mem.load(data)
    }
    pub fn load_test_v(&mut self, data:Vec<u8>){
        for (i, v) in data.iter().enumerate(){
            self.cpu.set_v(*v, i);
        }
    }

    pub fn test_eq_v(&self, vx:usize, val :u8){
        let vx_val = self.cpu.get_v(vx);
        assert_eq!(vx_val,val);
    }

}
