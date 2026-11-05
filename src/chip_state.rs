use crate::OpCode;

/// Two's complement add; the carry is the carry out of bit 15 (unsigned view).
fn add_with_carry(a: i16, b: i16) -> (i16, bool) {
    let (result, carry) = (a as u16).overflowing_add(b as u16);
    (result as i16, carry)
}

/// Two's complement subtract; the borrow is set when a < b as unsigned values.
fn sub_with_borrow(a: i16, b: i16) -> (i16, bool) {
    let (result, borrow) = (a as u16).overflowing_sub(b as u16);
    (result as i16, borrow)
}

pub struct ChipState {
    registers: [i16; 8],
    program_counter: usize,
    exception_program_counter: usize,
    stack_pointer: i32,
    instruction_memory: [u16; 4096],
    data_memory: [i16; 4096],
    ccr: [bool; 3],
    port_out: i16,
    port_in: i16,
    reserved_ccr: [bool; 3],
    pc_written: bool,
}

impl ChipState {
    pub fn new(instruction_memory: [u16; 4096], port_in: i16) -> Self {
        let starting_address = instruction_memory[0];
        Self {
            registers: [0; 8],
            program_counter: starting_address as usize,
            exception_program_counter: 0,
            stack_pointer: 2_i32.pow(12) - 1,
            instruction_memory,
            data_memory: [0; 4096],
            ccr: [false; 3],
            reserved_ccr: [false; 3],
            port_in,
            port_out: 0,
            pc_written: false,
        }
    }

    fn trigger_exception(&mut self, exception_address: usize) {
        self.exception_program_counter = self.program_counter.clamp(0, 4095);
        self.program_counter = self.instruction_memory[exception_address] as usize;
        self.pc_written = true;
    }

    fn trigger_invalid_memory_address_exception(&mut self) {
        println!("Program triggered an invalid memeory address exception");
        self.trigger_exception(2);
    }

    fn trigger_stack_underflow_exception(&mut self) {
        println!("Program triggered a stack underflow exception");
        self.trigger_exception(1);
    }

    fn set_program_counter(&mut self, new_value: usize) -> bool {
        if new_value > 2_usize.pow(12) - 1 {
            self.trigger_invalid_memory_address_exception();
            return false;
        }
        self.program_counter = new_value;
        self.pc_written = true;
        true
    }

    pub fn starting_address(&self) -> usize {
        self.instruction_memory[0] as usize
    }

    pub fn reset(&mut self) {
        self.registers = [0; 8];
        self.program_counter = self.starting_address();
        self.exception_program_counter = 0;
        self.stack_pointer = 2_i32.pow(12) - 1;
        self.ccr = [false; 3];
        self.pc_written = false;
    }

    fn push_value(&mut self, value: i16) -> bool {
        if self.stack_pointer < 0 || self.stack_pointer > 4095 {
            self.trigger_invalid_memory_address_exception();
            return false;
        }
        self.data_memory[self.stack_pointer as usize] = value;
        self.stack_pointer -= 1;
        true
    }

    fn push(&mut self, rsrc1: u16) {
        if !(0..=7).contains(&rsrc1) {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        self.push_value(self.registers[rsrc1 as usize]);
    }

    fn pop(&mut self, rdst: u16) {
        if !(0..=7).contains(&rdst) {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        if self.stack_pointer >= 2_i32.pow(12) - 1 {
            self.trigger_stack_underflow_exception();
            return;
        }
        self.stack_pointer += 1;
        self.registers[rdst as usize] = self.data_memory[self.stack_pointer as usize];
    }

    fn setc(&mut self) {
        self.ccr[2] = true;
    }

    fn not(&mut self, rdst: u16, rsrc1: u16) {
        if rdst > 7 || rsrc1 > 7 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let result = !self.registers[rsrc1 as usize];
        self.registers[rdst as usize] = result;
        self.ccr[0] = result == 0;
        self.ccr[1] = result < 0;
    }

    fn inc(&mut self, rdst: u16, rsrc1: u16) {
        if rdst > 7 || rsrc1 > 7 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let val = self.registers[rsrc1 as usize];
        let (result, carry) = add_with_carry(val, 1);

        self.registers[rdst as usize] = result;
        self.ccr[0] = result == 0; // Z flag
        self.ccr[1] = result < 0; // N flag
        self.ccr[2] = carry; // C flag (carry out of bit 15)
    }

    fn out(&mut self, rsrc1: u16) {
        if !(0..=7).contains(&rsrc1) {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        self.port_out = self.registers[rsrc1 as usize];
        println!("Program outputed: {}", self.port_out);
    }

    fn input(&mut self, rdst: u16) {
        if !(0..=7).contains(&rdst) {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        self.registers[rdst as usize] = self.port_in;
    }

    fn mov(&mut self, rdst: u16, rsrc1: u16) {
        if !(0..=7).contains(&rdst) || !(0..=7).contains(&rsrc1) {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        self.registers[rdst as usize] = self.registers[rsrc1 as usize];
    }

    fn add(&mut self, rdst: u16, rsrc1: u16, rsrc2: u16) {
        if !(0..=7).contains(&rdst) || !(0..=7).contains(&rsrc1) || !(0..=7).contains(&rsrc2) {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let val1 = self.registers[rsrc1 as usize];
        let val2 = self.registers[rsrc2 as usize];
        let (result, carry) = add_with_carry(val1, val2);

        self.registers[rdst as usize] = result;
        self.ccr[0] = result == 0; // Z flag
        self.ccr[1] = result < 0; // N flag
        self.ccr[2] = carry; // C flag
    }

    fn sub(&mut self, rdst: u16, rsrc1: u16, rsrc2: u16) {
        if !(0..=7).contains(&rdst) || !(0..=7).contains(&rsrc1) || !(0..=7).contains(&rsrc2) {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let val1 = self.registers[rsrc1 as usize];
        let val2 = self.registers[rsrc2 as usize];
        let (result, borrow) = sub_with_borrow(val1, val2);

        self.registers[rdst as usize] = result;
        self.ccr[0] = result == 0; // Z flag
        self.ccr[1] = result < 0; // N flag
        self.ccr[2] = borrow; // C flag (set on borrow, i.e. rsrc1 < rsrc2 unsigned)
    }

    fn and(&mut self, rdst: u16, rsrc1: u16, rsrc2: u16) {
        if !(0..=7).contains(&rdst) || !(0..=7).contains(&rsrc1) || !(0..=7).contains(&rsrc2) {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let val1 = self.registers[rsrc1 as usize];
        let val2 = self.registers[rsrc2 as usize];
        let result = val1 & val2;

        self.registers[rdst as usize] = result;
        self.ccr[0] = result == 0; // Z flag
        self.ccr[1] = result < 0; // N flag
    }

    fn iadd(&mut self, rdst: u16, rsrc1: u16, imm: i16) {
        if !(0..=7).contains(&rdst) || !(0..=7).contains(&rsrc1) {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let val1 = self.registers[rsrc1 as usize];
        let (result, carry) = add_with_carry(val1, imm);

        self.registers[rdst as usize] = result;
        self.ccr[0] = result == 0; // Z flag
        self.ccr[1] = result < 0; // N flag
        self.ccr[2] = carry; // C flag
    }

    fn ldm(&mut self, rdst: u16, imm: i16) {
        if !(0..=7).contains(&rdst) {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        self.registers[rdst as usize] = imm;
    }

    fn ldd(&mut self, rdst: u16, rsrc1: u16, offset: i16) {
        if !(0..=7).contains(&rdst) || !(0..=7).contains(&rsrc1) {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let base = self.registers[rsrc1 as usize];
        let target_address = base.wrapping_add(offset) as u16 as usize;

        if target_address >= 4096 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        self.registers[rdst as usize] = self.data_memory[target_address];
    }

    fn std(&mut self, rsrc1: u16, rsrc2: u16, offset: i16) {
        if !(0..=7).contains(&rsrc1) || !(0..=7).contains(&rsrc2) {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let base = self.registers[rsrc2 as usize];
        let target_address = base.wrapping_add(offset) as u16 as usize;

        if target_address >= 4096 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        self.data_memory[target_address] = self.registers[rsrc1 as usize];
    }

    fn jz(&mut self, rsrc1: u16) {
        if rsrc1 > 7 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        if self.ccr[0] {
            self.ccr[0] = false;
            let value = self.registers[rsrc1 as usize];

            if value < 0 {
                self.trigger_invalid_memory_address_exception();
                return;
            }
            self.set_program_counter(value as usize);
        }
    }

    fn jn(&mut self, rsrc1: u16) {
        if rsrc1 > 7 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        if self.ccr[1] {
            self.ccr[1] = false;
            let value = self.registers[rsrc1 as usize];

            if value < 0 {
                self.trigger_invalid_memory_address_exception();
                return;
            }
            self.set_program_counter(value as usize);
        }
    }
    fn jc(&mut self, rsrc1: u16) {
        if rsrc1 > 7 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        if self.ccr[2] {
            self.ccr[2] = false;
            let value = self.registers[rsrc1 as usize];

            if value < 0 {
                self.trigger_invalid_memory_address_exception();
                return;
            }
            self.set_program_counter(value as usize);
        }
    }

    fn jmp(&mut self, rsrc1: u16) {
        if rsrc1 > 7 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let value = self.registers[rsrc1 as usize];

        if value < 0 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        self.set_program_counter(value as usize);
    }

    fn call(&mut self, rsrc1: u16) {
        if rsrc1 > 7 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        let value = self.registers[rsrc1 as usize];

        if value < 0 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        if self.push_value((self.program_counter + 1) as i16) {
            self.set_program_counter(value as usize);
        }
    }

    fn ret(&mut self) {
        if self.stack_pointer >= 2_i32.pow(12) - 1 {
            self.trigger_stack_underflow_exception();
            return;
        }
        self.stack_pointer += 1;

        let value = self.data_memory[self.stack_pointer as usize];

        if value < 0 {
            self.trigger_invalid_memory_address_exception();
            return;
        }

        self.set_program_counter(value as usize);
    }

    fn int(&mut self, index: u16) {
        if index > 1 {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        let entry = self.instruction_memory[3] as usize + index as usize;
        if entry > 0x0FFF {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        let handler = self.instruction_memory[entry] as usize;
        if handler > 0x0FFF {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        if self.push_value((self.program_counter + 1) as i16) {
            self.reserved_ccr = self.ccr;
            self.set_program_counter(handler);
        }
    }

    fn rti(&mut self) {
        if self.stack_pointer >= 2_i32.pow(12) - 1 {
            self.trigger_stack_underflow_exception();
            return;
        }
        self.stack_pointer += 1;

        let value = self.data_memory[self.stack_pointer as usize];
        if value < 0 {
            self.trigger_invalid_memory_address_exception();
            return;
        }
        self.set_program_counter(value as usize);

        self.ccr = self.reserved_ccr;
    }

    fn parse_opcode(&mut self, opcode: &OpCode) {
        match opcode {
            OpCode::NOP => (),
            OpCode::HLT => (), // Will never reach here but it is there to satisfy the compiler
            OpCode::SETC => self.setc(),
            OpCode::RET => self.ret(),
            OpCode::RTI => self.rti(),
            OpCode::NOT(rdst, rsrc1) => self.not(*rdst, *rsrc1),
            OpCode::INC(rdst, rsrc1) => self.inc(*rdst, *rsrc1),
            OpCode::OUT(rsrc1) => self.out(*rsrc1),
            OpCode::IN(rdst) => self.input(*rdst),
            OpCode::PUSH(rsrc1) => self.push(*rsrc1),
            OpCode::POP(rdst) => self.pop(*rdst),
            OpCode::JZ(rsrc1) => self.jz(*rsrc1),
            OpCode::JN(rsrc1) => self.jn(*rsrc1),
            OpCode::JC(rsrc1) => self.jc(*rsrc1),
            OpCode::JMP(rsrc1) => self.jmp(*rsrc1),
            OpCode::CALL(rsrc1) => self.call(*rsrc1),
            OpCode::INT(index) => self.int(*index),
            OpCode::MOV(rdst, rsrc1) => self.mov(*rdst, *rsrc1),
            OpCode::ADD(rdst, rsrc1, rsrc2) => self.add(*rdst, *rsrc1, *rsrc2),
            OpCode::SUB(rdst, rsrc1, rsrc2) => self.sub(*rdst, *rsrc1, *rsrc2),
            OpCode::AND(rdst, rsrc1, rsrc2) => self.and(*rdst, *rsrc1, *rsrc2),
            OpCode::IADD(rdst, rsrc1, imm) => self.iadd(*rdst, *rsrc1, *imm),
            OpCode::LDM(rdst, imm) => self.ldm(*rdst, *imm),
            OpCode::LDD(rdst, rsrc1, offset) => self.ldd(*rdst, *rsrc1, *offset),
            OpCode::STD(rsrc1, rsrc2, offset) => self.std(*rsrc1, *rsrc2, *offset),
        };
    }

    fn consume_unsigned_word(&mut self, current_counter: &mut usize) -> u16 {
        let value = self.instruction_memory[*current_counter];
        *current_counter += 1;
        value
    }

    fn consume_signed_word(&mut self, current_counter: &mut usize) -> i16 {
        let value = self.instruction_memory[*current_counter] as i16;
        *current_counter += 1;
        value
    }

    pub fn decode_next_opcode(&mut self) -> (OpCode, usize) {
        if self.program_counter >= 4096 {
            self.trigger_invalid_memory_address_exception();
            return (OpCode::NOP, self.program_counter);
        }

        let mut current_counter: usize = self.program_counter;

        // Layout 7 bits: opcodes | 9 bits registers, rdst, rsrc1, rsrc2 in that order
        let first_word = self.consume_unsigned_word(&mut current_counter);
        let opcode = (first_word & 0b1111_1110_0000_0000) >> 9;
        let rdst = (first_word & 0b0000_0001_1100_0000) >> 6;
        let rsrc1 = (first_word & 0b0000_0000_0011_1000) >> 3;
        let rsrc2 = first_word & 0b0000_0000_0000_0111;
        let mut extra_value: Option<i16> = None;

        if opcode > 20 && opcode < 25 {
            if current_counter >= 4096 {
                self.trigger_invalid_memory_address_exception();
                return (OpCode::NOP, self.program_counter);
            }

            let extra_value_value = self.consume_signed_word(&mut current_counter);
            extra_value = Some(extra_value_value);
        }

        let value = match opcode {
            0 => OpCode::NOP,
            1 => OpCode::HLT,
            2 => OpCode::SETC,
            3 => OpCode::NOT(rdst, rsrc1),
            4 => OpCode::INC(rdst, rsrc1),
            5 => OpCode::OUT(rsrc1),
            6 => OpCode::IN(rdst),
            7 => OpCode::MOV(rdst, rsrc1),
            8 => OpCode::ADD(rdst, rsrc1, rsrc2),
            9 => OpCode::SUB(rdst, rsrc1, rsrc2),
            10 => OpCode::AND(rdst, rsrc1, rsrc2),
            11 => OpCode::PUSH(rsrc1),
            12 => OpCode::POP(rdst),
            13 => OpCode::JZ(rsrc1),
            14 => OpCode::JN(rsrc1),
            15 => OpCode::JC(rsrc1),
            16 => OpCode::JMP(rsrc1),
            17 => OpCode::CALL(rsrc1),
            18 => OpCode::RET,
            19 => {
                let index = first_word & 0b0000_0000_0000_0001;
                OpCode::INT(index)
            }
            20 => OpCode::RTI,
            21 => OpCode::IADD(rdst, rsrc1, extra_value.unwrap()),
            22 => OpCode::LDM(rdst, extra_value.unwrap()),
            23 => OpCode::LDD(rdst, rsrc1, extra_value.unwrap()),
            24 => OpCode::STD(rsrc1, rsrc2, extra_value.unwrap()),
            _ => OpCode::NOP,
        };

        (value, current_counter)
    }

    pub fn cycle(&mut self) -> bool {
        let (opcode, new_pc) = self.decode_next_opcode();

        if let OpCode::HLT = opcode {
            return false;
        }

        self.parse_opcode(&opcode);
        if !self.pc_written {
            self.program_counter = new_pc;
        } else {
            self.pc_written = false;
        }
        true
    }
}
