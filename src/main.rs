use std::collections::BTreeMap;
use std::num::Wrapping;
use text_io::read;



// ---------- INSTRUCTIONS ----------------------------------------------------

pub enum OpCode {
    CMOV = 0x0,
    LOAD = 0x1,
    STORE = 0x2,
    ADD = 0x3,
    MUL = 0x4,
    DIV = 0x5,
    NAND = 0x6,
    HALT = 0x7,
    ALLOC = 0x8,
    FREE = 0x9,
    OUT = 0xA,
    IN = 0xB,
    CALL = 0xC,
    CONST = 0xD,
}

impl OpCode {
    pub fn from_byte(b: u8) -> OpCode {
        match b {
            0x0 => OpCode::CMOV,
            0x1 => OpCode::LOAD,
            0x2 => OpCode::STORE,
            0x3 => OpCode::ADD,
            0x4 => OpCode::MUL,
            0x5 => OpCode::DIV,
            0x6 => OpCode::NAND,
            0x7 => OpCode::HALT,
            0x8 => OpCode::ALLOC,
            0x9 => OpCode::FREE,
            0xA => OpCode::OUT,
            0xB => OpCode::IN,
            0xC => OpCode::CALL,
            0xD => OpCode::CONST,
            _ => {
                panic!("Encountered invalid instruction!");
            }
        }
    }
}

type Register = u8; // 4 bit number
type Data = u32;
type PlatterIndex = u64;

#[inline(always)]
fn upper_byte(data: Data) -> Register {
    ((data >> 28) & 0b1111) as Register
}

#[inline(always)]
fn upper_reg(data: Data) -> Register {
    ((data >> 25) & 0b111) as Register
}

#[inline(always)]
fn upper_val(data: Data) -> Data {
    data & 0x1FFFFFF
}

#[inline(always)]
fn parse_r_a(data: Data) -> Register {
    ((data >> 6) & 0b111) as Register
}

#[inline(always)]
fn parse_r_b(data: Data) -> Register {
    ((data >> 3) & 0b111) as Register
}

#[inline(always)]
fn parse_r_c(data: Data) -> Register {
    (data & 0b111) as Register
}

pub struct Instruction {
    op_code: OpCode,
    r_a: Register,
    r_b: Register,
    r_c: Register,
    value: Data,
}

impl Instruction {
    pub fn decode(data: Data) -> Self {
        let op: OpCode = OpCode::from_byte(upper_byte(data));
        match op {
            OpCode::CMOV
            | OpCode::LOAD
            | OpCode::STORE
            | OpCode::ADD
            | OpCode::MUL
            | OpCode::DIV
            | OpCode::NAND
            | OpCode::HALT
            | OpCode::ALLOC
            | OpCode::FREE
            | OpCode::OUT
            | OpCode::IN
            | OpCode::CALL => Instruction {
                op_code: op,
                r_a: parse_r_a(data),
                r_b: parse_r_b(data),
                r_c: parse_r_c(data),
                value: 0,
            },

            OpCode::CONST => Instruction {
                op_code: OpCode::CONST,
                r_a: upper_reg(data),
                r_b: 0,
                r_c: 0,
                value: upper_val(data),
            },
        }
    }
}

// ---------- CPU EMULATION ---------------------------------------------------

type Platter = Vec<Data>;

#[derive(Debug)]
pub struct CPU {
    status: bool,
    register_file: [Data; 8],
    instruction_pointer: PlatterIndex,
    instruction_platter: Vec<Data>,
    memory: BTreeMap<Data, Platter>,
    next_allocate: Data,
}

impl CPU {
    pub fn new(program: Vec<Data>) -> Self {
        CPU {
            status: true,
            register_file: [0; 8],
            instruction_pointer: 0x0,
            instruction_platter: program, // always platter 0, always active
            memory: BTreeMap::new(),
            next_allocate: 1,
        }
    }

    pub fn interpret(&mut self) {
        loop {
            // 1. Fetch Decode
            if !self.status {
                break;
            }
            let instruction = Instruction::decode(CPU::fetch_instruction(self));

            // 2. Regsiter/Execute
            self.execute_instruction(instruction);
        }
    }

    fn fetch_instruction(&mut self) -> Data {
        let data = self.instruction_platter[self.instruction_pointer as usize];
        self.instruction_pointer += 1;
        data
    }

    fn execute_instruction(&mut self, inst: Instruction) {
        match inst.op_code {
            OpCode::CMOV => {
                if self.register_file[inst.r_c as usize] != 0 {
                    self.register_file[inst.r_a as usize] = self.register_file[inst.r_b as usize]
                }
            }
            OpCode::LOAD => {
                let b = self.register_file[inst.r_b as usize];
                let c = self.register_file[inst.r_c as usize];
                let loaded: Data = if b == 0 {
                    self.instruction_platter[c as usize]
                } else {
                    let found = self.memory.get(&b);
                    match found {
                        None => {
                            panic!("Loaded inactive array");
                        }
                        Some(p) => p[c as usize],
                    }
                };
                self.register_file[inst.r_a as usize] = loaded;
            }
            OpCode::STORE => {
                let a = self.register_file[inst.r_a as usize];
                let b = self.register_file[inst.r_b as usize];
                let c = self.register_file[inst.r_c as usize];
                if a == 0 {
                    self.instruction_platter[b as usize] = c;
                } else {
                    let found = self.memory.get_mut(&a);
                    match found {
                        None => {
                            panic!("Stored to inactive array");
                        }
                        Some(p) => {
                            p[b as usize] = c;
                        }
                    }
                }
            }
            OpCode::ADD => {
                let b = Wrapping(self.register_file[inst.r_b as usize]);
                let c = Wrapping(self.register_file[inst.r_c as usize]);
                self.register_file[inst.r_a as usize] = (b + c).0;
            }
            OpCode::MUL => {
                let b = Wrapping(self.register_file[inst.r_b as usize]);
                let c = Wrapping(self.register_file[inst.r_c as usize]);
                self.register_file[inst.r_a as usize] = (b * c).0;
            }
            OpCode::DIV => {
                let b = Wrapping(self.register_file[inst.r_b as usize]);
                let c = Wrapping(self.register_file[inst.r_c as usize]);
                self.register_file[inst.r_a as usize] = (b / c).0;
            }
            OpCode::NAND => {
                let b = self.register_file[inst.r_b as usize];
                let c = self.register_file[inst.r_c as usize];
                self.register_file[inst.r_a as usize] = !(b & c);
            }
            OpCode::HALT => {
                self.status = false;
            }
            OpCode::ALLOC => {
                let c = self.register_file[inst.r_c as usize];

                let insert_result = self.memory.insert(self.next_allocate, vec![0; c as usize]);
                if let Some(_) = insert_result {
                    panic!("Problem in allocating new platter. Out of space?");
                }
                self.register_file[inst.r_b as usize] = self.next_allocate;

                // TODO: Slow. Refactor to be efficient and to handle out of space and 0
                self.next_allocate += 1;
                while self.memory.contains_key(&self.next_allocate) {
                    self.next_allocate += 1;
                }
            }
            OpCode::FREE => {
                let c = self.register_file[inst.r_c as usize];
                if c == 0 {
                    panic!("Cannot free program data");
                }

                let remove_result = self.memory.remove(&c);
                if let None = remove_result {
                    panic!("Double free");
                }
            }
            OpCode::OUT => {
                let c = self.register_file[inst.r_c as usize];
                if c > 255 {
                    panic!("Printed character out of bounds");
                }
                print!("{}", (c as u8) as char);
            }
            OpCode::IN => {
                let x: u8 = read!();
                let xn: i8 = x as i8;
                self.register_file[inst.r_c as usize] = (xn as i32) as Data;
            }
            OpCode::CALL => {
                let b = self.register_file[inst.r_b as usize];
                let c = self.register_file[inst.r_c as usize];

                if b != 0 {
                    let found = self.memory.get(&b);
                    match found {
                        None => {
                            panic!("Called inactive array");
                        }
                        Some(p) => {
                            self.instruction_platter = p.clone();
                        }
                    }
                }
                self.instruction_pointer = c as PlatterIndex;
            }
            OpCode::CONST => {
                self.register_file[inst.r_a as usize] = inst.value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_halt() {
        let program: Vec<Data> = vec![0x70000000];
        let mut cpu = CPU::new(program);
        cpu.interpret();
        assert!(cpu.instruction_pointer == 1);
    }

    #[test]
    #[should_panic]
    fn invalid_instruction_e() {
        let program: Vec<Data> = vec![0xE0000000];
        CPU::new(program).interpret();
    }

    #[test]
    #[should_panic]
    fn invalid_instruction_f() {
        let program: Vec<Data> = vec![0xF0000000];
        CPU::new(program).interpret();
    }
    #[test]
    #[should_panic]
    fn end_of_platter() {
        let program: Vec<Data> = vec![];
        CPU::new(program).interpret();
    }

    #[test]
    fn cmov_no_move() {
        let program: Vec<Data> = vec![0b00000000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0xDEADBEEF;
        cpu.register_file[1] = 0xDECAF000;
        cpu.register_file[2] = 0x0;
        cpu.interpret();
        assert!(cpu.register_file[0] == 0xDEADBEEF);
        assert!(cpu.register_file[1] == 0xDECAF000);
        assert!(cpu.register_file[2] == 0x0);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn cmov_move() {
        let program: Vec<Data> = vec![0b00000000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0xDEADBEEF;
        cpu.register_file[1] = 0xDECAF000;
        cpu.register_file[2] = 0x1;
        cpu.interpret();
        assert!(cpu.register_file[0] == 0xDECAF000);
        assert!(cpu.register_file[1] == 0xDECAF000);
        assert!(cpu.register_file[2] == 0x1);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn cmov_move_to_self() {
        let program: Vec<Data> = vec![0b00000000000000000000000011011010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[3] = 0xDEADBEEF;
        cpu.register_file[2] = 0x1;
        cpu.interpret();
        assert!(cpu.register_file[3] == 0xDEADBEEF);
        assert!(cpu.register_file[2] == 0x1);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn add_no_wrap() {
        let program: Vec<Data> = vec![0b00110000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0;
        cpu.register_file[1] = 0x1;
        cpu.register_file[2] = 0x7;
        cpu.interpret();
        assert!(cpu.register_file[0] == 0x8);
        assert!(cpu.register_file[1] == 0x1);
        assert!(cpu.register_file[2] == 0x7);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn add_wrap() {
        let program: Vec<Data> = vec![0b00110000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0;
        cpu.register_file[1] = 0x1;
        cpu.register_file[2] = u32::MAX;
        cpu.interpret();
        assert!(cpu.register_file[0] == 0x0);
        assert!(cpu.register_file[1] == 0x1);
        assert!(cpu.register_file[2] == u32::MAX);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn mul_no_wrap() {
        let program: Vec<Data> = vec![0b01000000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0;
        cpu.register_file[1] = 2;
        cpu.register_file[2] = 7;
        cpu.interpret();
        assert!(cpu.register_file[0] == 14);
        assert!(cpu.register_file[1] == 2);
        assert!(cpu.register_file[2] == 7);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn mul_wrap() {
        let program: Vec<Data> = vec![0b01000000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0;
        cpu.register_file[1] = u32::MAX;
        cpu.register_file[2] = 2;
        cpu.interpret();
        assert!(cpu.register_file[0] == u32::MAX - 1);
        assert!(cpu.register_file[1] == u32::MAX);
        assert!(cpu.register_file[2] == 2);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn div_even() {
        let program: Vec<Data> = vec![0b01010000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0;
        cpu.register_file[1] = 8;
        cpu.register_file[2] = 2;
        cpu.interpret();
        assert!(cpu.register_file[0] == 4);
        assert!(cpu.register_file[1] == 8);
        assert!(cpu.register_file[2] == 2);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    fn div_round() {
        let program: Vec<Data> = vec![0b01010000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0;
        cpu.register_file[1] = 8;
        cpu.register_file[2] = 3;
        cpu.interpret();
        assert!(cpu.register_file[0] == 2);
        assert!(cpu.register_file[1] == 8);
        assert!(cpu.register_file[2] == 3);
        assert!(cpu.instruction_pointer == 2);
    }

    #[test]
    #[should_panic]
    fn div_0() {
        let program: Vec<Data> = vec![0b01010000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 1;
        cpu.register_file[1] = 8;
        cpu.register_file[2] = 0;
        cpu.interpret();
    }

    #[test]
    fn single_allocate() {
        let program: Vec<Data> = vec![0b10000000000000000000000000000001, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[1] = 17;
        cpu.interpret();
        assert!(cpu.memory.len() == 1);
        let allocated_platter = cpu.memory.get(&cpu.register_file[0]);
        match allocated_platter {
            None => {
                panic!("fail");
            }
            Some(p) => {
                assert!(p.len() == 17);
            }
        }
    }

    #[test]
    fn two_allocate() {
        let program: Vec<Data> = vec![
            0b10000000000000000000000000000001,
            0b00000000000000000000000111000001, // MOVE r2 -> r7
            0b10000000000000000000000000000010,
            0x70000000,
        ];
        let mut cpu = CPU::new(program);
        cpu.register_file[1] = 12;
        cpu.register_file[2] = 17;
        cpu.interpret();

        assert!(cpu.memory.len() == 2);

        let allocated_platter_1 = cpu.memory.get(&cpu.register_file[7]);
        match allocated_platter_1 {
            None => {
                panic!("fail");
            }
            Some(p) => {
                assert!(p.len() == 12);
            }
        }

        let allocated_platter_2 = cpu.memory.get(&cpu.register_file[0]);
        match allocated_platter_2 {
            None => {
                panic!("fail");
            }
            Some(p) => {
                assert!(p.len() == 17);
            }
        }
    }
    #[test]
    fn allocate_then_free() {
        let program: Vec<Data> = vec![
            0b10000000000000000000000000001010,
            0b10010000000000000000000000000001,
            0x70000000,
        ];
        let mut cpu = CPU::new(program);
        cpu.register_file[2] = 17;
        cpu.interpret();
        assert!(cpu.memory.len() == 0);
        let allocated_platter_1 = cpu.memory.get(&cpu.register_file[1]);
        if let Some(_) = allocated_platter_1 {
            panic!("fail");
        }
    }

    #[test]
    fn test_nand() {
        let program: Vec<Data> = vec![0b01100000000000000000000000001010, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0x0;
        cpu.register_file[1] = 0xFFFF00FF;
        cpu.register_file[2] = 0xFFFF0F0F;
        cpu.interpret();
        assert!(cpu.register_file[0] == 0xFFF0);
    }

    #[test]
    fn test_alloc_store_load() {
        let program: Vec<Data> = vec![
            0b10000000000000000000000000111000, // r7 <- ALLOC (r0)
            0b00100000000000000000000111001010, // MEM(r7)[r1] <- r2
            0b00010000000000000000000110111001, // r6 <- MEM(r7)[r1]
            0x70000000,
        ];
        let mut cpu = CPU::new(program);

        cpu.register_file[0] = 20;
        cpu.register_file[1] = 5;
        cpu.register_file[2] = 0xDEADBEEF;

        cpu.interpret();
        assert!(cpu.register_file[6] == 0xDEADBEEF);
    }

    #[test]
    #[should_panic]
    fn test_store_unallocated_space() {
        let program: Vec<Data> = vec![
            0b00100000000000000000000111001010, // MEM(r7)[r1] <- r2
            0x70000000,
        ];
        let mut cpu = CPU::new(program);
        cpu.register_file[7] = 100;
        cpu.interpret();
    }

    #[test]
    fn test_store_program_memory() {
        let program: Vec<Data> = vec![
            0b00100000000000000000000111001010, // MEM(r7)[r1] <- r2
            0b00010000000000000000000110111001, // r6 <- MEM(r7)[r1]
            0x70000000,
        ];
        let mut cpu = CPU::new(program);
        cpu.register_file[1] = 0;
        cpu.register_file[7] = 0;
        cpu.register_file[2] = 0xDEADBEEF;
        cpu.interpret();
        assert!(cpu.register_file[6] == 0xDEADBEEF);
    }

    #[test]
    #[should_panic]
    fn test_load_unallocated_space() {
        let program: Vec<Data> = vec![
            0b00010000000000000000000110111001, // r6 <- MEM(r7)[r1]
            0x70000000,
        ];
        let mut cpu = CPU::new(program);
        cpu.register_file[7] = 100;
        cpu.interpret();
    }

    #[test]
    fn print_c() {
        let program: Vec<Data> = vec![0b10100000000000000000000000000111, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[7] = 67;
        cpu.interpret();
    }

    #[test]
    fn print_0() {
        let program: Vec<Data> = vec![0b10100000000000000000000000000111, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[7] = 0;
        cpu.interpret();
    }

    #[test]
    fn print_255() {
        let program: Vec<Data> = vec![0b10100000000000000000000000000111, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[7] = 255;
        cpu.interpret();
    }

    #[test]
    #[should_panic]
    fn print_out_of_bounds() {
        let program: Vec<Data> = vec![0b10100000000000000000000000000111, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.register_file[7] = 256;
        cpu.interpret();
    }

    #[test]
    fn constant_load() {
        let program: Vec<Data> = vec![0b11011110101010101101001001010101, 0x70000000];
        let mut cpu = CPU::new(program);
        cpu.interpret();
        assert!(cpu.register_file[7] == 0b0101010101101001001010101);
    }


    #[test]
    fn call_array() {
        let program: Vec<Data> = vec![
            0b10000000000000000000000000101000,     // r5 <- ALLOC(r0)   size 5
            0b00100000000000000000000101001010,     // MEM(r5)[r1] = r2   (halt)
            0b11000000000000000000000000101001];    // CALL(r5)[r1]
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 5;           // Size of new array
        cpu.register_file[1] = 3;           // Index in new array to jump to
        cpu.register_file[2] = 0x70000000;  // HALT
        cpu.interpret();
        // If the program halts, it sucessfully copied the HALT instruciton and 
        //  jumped to it. 
    }


    #[test]
    #[should_panic]
    fn call_inactive_array() {
        let program: Vec<Data> = vec![
            0b11000000000000000000000000101001];    // CALL(r5)[r1]
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 5;           // Size of new array
        cpu.register_file[1] = 3;           // Index in new array to jump to
        cpu.register_file[2] = 0x70000000;  // HALT
        cpu.interpret();
    }


    #[test]
    fn leap_of_faith() {
        // Jump within the instruction platter by calling array 0
        let program: Vec<Data> = vec![
            0b11000000000000000000000000000001, // CALL(r0)[r1]   r0=0 r1=4
            0b11111111111111111111111111111111, // INVALID
            0b11111111111111111111111111111111, // INVALID
            0b11111111111111111111111111111111, // INVALID
            0x70000000];                        // HALT
        let mut cpu = CPU::new(program);
        cpu.register_file[0] = 0;
        cpu.register_file[1] = 4;
        cpu.interpret();
    }

    // #[test]
    // fn input_test() {
    //     let program: Vec<Data> = vec![0b10110000000000000000000111010111, 0x70000000];
    //     let mut cpu = CPU::new(program);
    //     cpu.interpret();
    //     println!(
    //         "{} {} {} {} {} {} {} {}",
    //         cpu.register_file[0],
    //         cpu.register_file[1],
    //         cpu.register_file[2],
    //         cpu.register_file[3],
    //         cpu.register_file[4],
    //         cpu.register_file[5],
    //         cpu.register_file[6],
    //         cpu.register_file[7]
    //     );
    // }
}


fn u8x4_to_u32_big_endian(u8s: &[u8]) -> u32{
    ((u8s[0] as u32) << 24) 
        + ((u8s[1] as u32) << 16) 
        + ((u8s[2] as u32) << 8)
        + (u8s[3] as u32)
}

pub fn main() {
    let codex_raw: Vec<u8> = std::fs::read("./codex.umz").unwrap();
    let codex: Vec<u32> 
        = codex_raw
            .chunks(4)
            .map(|u8s|u8x4_to_u32_big_endian(u8s))
            .collect::<Vec<u32>>();
    
    let mut cpu = CPU::new(codex);
    cpu.interpret();
}
