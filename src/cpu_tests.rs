#[cfg(test)]
mod tests {
    use crate::system;
    use std::collections::HashMap;

    // SETUP
    fn vx_test(instruction: u16, val: u8) -> system::System {
        let mut emu = system::System::new();
        emu.load_test(vec![
            ((instruction & 0xFF00) >> 8) as u8,
            (instruction & 0x00FF) as u8,
        ]);
        emu.load_test_v(vec![
            0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF,
        ]);
        emu.tick(&keys_setup());
        emu.test_eq_v(0x0, val);
        emu
    }
    fn keys_setup() -> HashMap<sdl2::keyboard::Keycode, i32> {
        let keys: HashMap<sdl2::keyboard::Keycode, i32> = HashMap::new();
        keys
    }
    // VX TESTS

    #[test]
    fn vx_set_vy() {
        vx_test(0x8020, 0x3);
    }
    #[test]
    fn vx_bit_or_vy() {
        vx_test(0x8021, 0x3);
    }
    #[test]
    fn vx_bit_and_vy() {
        vx_test(0x8022, 0x1);
    }

    #[test]
    fn vx_bit_x_or_vy() {
        vx_test(0x8023, 0x2);
    }
    #[test]
    fn vx_bit_add_vy() {
        vx_test(0x8024, 0x4).test_eq_v(0xF, 0);
    }

    #[test]
    fn vx_sub_vy() {
        vx_test(0x8025, 0xFE);
    }

    #[test]
    fn vx_bit_shift_r_vy(){
        vx_test(0x8046, 0x0).test_eq_v(0xF,0x1);
    }

}
