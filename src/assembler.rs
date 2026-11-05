/* Structure of code:
 * Section_Label:
 * There are section labels:
 * - Stack_Underflow_Handler
 * - Invalid_Memory_Address_Handler
 * - Interrupt 0
 * - Interrupt 1
 * - Main
 * If any section is omitted that section will be consider a single HLT instruction
 *
 * Instructions are written as:
 * INS rdst rsrc1 rsrc2 imm offset
 * if any value is not needed by the instruction it is omitted and the next value takes its place
 * for example:
 * NOP
 * OUT rsrc1
 * IN rdst
 * ADD rdst rsrc1 rsrc2
 * IADD rdst rsrc1 imm
 *
 * Comments are writting using ;
 *
 * The binary will always look like:
 * 0: Address of Main
 * 1: Address of the stack underflow handler
 * 2: Address of the invalid memory address handler
 * 3: Address of the interrupt vtable
 * 4: Stack underflow handler
 * 5: Invalid memory address handler
 * 6: interrupt vtable
 * 7: interrupt 0
 * 8: interrupt 1
 * 9: Main
 * */

use crate::OpCode;

fn encode_opcode(opcode: &OpCode) -> (u16, Option<i16>) {
    let (opcode_num, rdst, rsrc1, rsrc2): (u16, u16, u16, u16) = match opcode {
        OpCode::NOP => (0, 0, 0, 0),
        OpCode::HLT => (1, 0, 0, 0),
        OpCode::SETC => (2, 0, 0, 0),
        OpCode::NOT(rdst, rsrc1) => (3, *rdst, *rsrc1, 0),
        OpCode::INC(rdst, rsrc1) => (4, *rdst, *rsrc1, 0),
        OpCode::OUT(rsrc1) => (5, 0, *rsrc1, 0),
        OpCode::IN(rdst) => (6, *rdst, 0, 0),
        OpCode::MOV(rdst, rsrc1) => (7, *rdst, *rsrc1, 0),
        OpCode::ADD(rdst, rsrc1, rsrc2) => (8, *rdst, *rsrc1, *rsrc2),
        OpCode::SUB(rdst, rsrc1, rsrc2) => (9, *rdst, *rsrc1, *rsrc2),
        OpCode::AND(rdst, rsrc1, rsrc2) => (10, *rdst, *rsrc1, *rsrc2),
        OpCode::PUSH(rsrc1) => (11, 0, *rsrc1, 0),
        OpCode::POP(rdst) => (12, *rdst, 0, 0),
        OpCode::JZ(rsrc1) => (13, 0, *rsrc1, 0),
        OpCode::JN(rsrc1) => (14, 0, *rsrc1, 0),
        OpCode::JC(rsrc1) => (15, 0, *rsrc1, 0),
        OpCode::JMP(rsrc1) => (16, 0, *rsrc1, 0),
        OpCode::CALL(rsrc1) => (17, 0, *rsrc1, 0),
        OpCode::RET => (18, 0, 0, 0),
        OpCode::INT(index) => (19, 0, 0, *index),
        OpCode::RTI => (20, 0, 0, 0),
        OpCode::IADD(rdst, rsrc1, _) => (21, *rdst, *rsrc1, 0),
        OpCode::LDM(rdst, _) => (22, *rdst, 0, 0),
        OpCode::LDD(rdst, rsrc1, _) => (23, *rdst, *rsrc1, 0),
        OpCode::STD(rsrc1, rsrc2, _) => (24, 0, *rsrc1, *rsrc2),
    };

    let mut extra_value: Option<i16> = None;

    if opcode_num > 20 && opcode_num < 25 {
        extra_value = match opcode {
            OpCode::IADD(_, _, value) => Some(*value),
            OpCode::LDM(_, value) => Some(*value),
            OpCode::LDD(_, _, value) => Some(*value),
            OpCode::STD(_, _, value) => Some(*value),
            _ => None,
        }
    }

    let opcode_num = (opcode_num << 9) | (rdst << 6) | (rsrc1 << 3) | rsrc2;

    (opcode_num, extra_value)
}

fn encode_opcodes(opcodes: &[OpCode]) -> Vec<u16> {
    let mut length = opcodes.len();
    for opcode in opcodes {
        if matches!(opcode, OpCode::IADD(..))
            || matches!(opcode, OpCode::LDM(..))
            || matches!(opcode, OpCode::LDD(..))
            || matches!(opcode, OpCode::STD(..))
        {
            length += 1;
        }
    }
    let mut bytes = Vec::with_capacity(length);

    for opcode in opcodes {
        let (first_byte, second_byte) = encode_opcode(opcode);
        bytes.push(first_byte);
        if let Some(second_byte) = second_byte {
            bytes.push(second_byte as u16);
        }
    }

    bytes
}

enum CodeFeature {
    OpCode(OpCode),
    Label(&'static str),
}

fn lex(lines: &[&str]) -> Result<Vec<CodeFeature>, String> {
    let length = lines
        .iter()
        .filter(|s| !s.starts_with(";"))
        .collect::<Vec<&&str>>()
        .len();
    let mut code_features: Vec<CodeFeature> = Vec::with_capacity(length);

    for (i, line) in lines.iter().enumerate() {
        let index_of_semicolon = line.find(';');
        let index_of_semicolon = index_of_semicolon.unwrap_or(line.len());
        let line = line[..index_of_semicolon].trim();

        if line.is_empty() {
            continue;
        }

        match line {
            "NOP" => code_features.push(CodeFeature::OpCode(OpCode::NOP)),
            "HLT" => code_features.push(CodeFeature::OpCode(OpCode::HLT)),
            "SETC" => code_features.push(CodeFeature::OpCode(OpCode::SETC)),
            "RET" => code_features.push(CodeFeature::OpCode(OpCode::RET)),
            "RTI" => code_features.push(CodeFeature::OpCode(OpCode::RTI)),
            "Stack_Underflow_Handler:" => {
                if code_features
                    .iter()
                    .any(|c| matches!(c, CodeFeature::Label("Stack_Underflow_Handler")))
                {
                    return Err(format!(
                        "Repeated Stack_Underflow_Handler label on line {}",
                        i
                    ));
                }
                code_features.push(CodeFeature::Label("Stack_Underflow_Handler"))
            }
            "Invalid_Memory_Address_Handler:" => {
                if code_features
                    .iter()
                    .any(|c| matches!(c, CodeFeature::Label("Invalid_Memory_Address_Handler")))
                {
                    return Err(format!(
                        "Repeated Invalid_Memory_Address_Handler label on line {}",
                        i
                    ));
                }
                code_features.push(CodeFeature::Label("Invalid_Memory_Address_Handler"))
            }
            "Interrupt 0:" => {
                if code_features
                    .iter()
                    .any(|c| matches!(c, CodeFeature::Label("Interrupt 0")))
                {
                    return Err(format!("Repeated Interrupt 0 label on line {}", i));
                }
                code_features.push(CodeFeature::Label("Interrupt 0"))
            }
            "Interrupt 1:" => {
                if code_features
                    .iter()
                    .any(|c| matches!(c, CodeFeature::Label("Interrupt 1")))
                {
                    return Err(format!("Repeated Interrupt 1 label on line {}", i));
                }
                code_features.push(CodeFeature::Label("Interrupt 1"))
            }
            "Main:" => {
                if code_features
                    .iter()
                    .any(|c| matches!(c, CodeFeature::Label("Main")))
                {
                    return Err(format!("Repeated Main label on line {}", i));
                }
                code_features.push(CodeFeature::Label("Main"))
            }
            _ => {
                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.len() == 2 {
                    let number: u16 = if tokens[0] == "INT" {
                        let Ok(parsed_number) = tokens[1].parse::<u16>() else {
                            return Err(format!("Invalid number on line {}", i));
                        };
                        if parsed_number > 1 {
                            return Err(format!("Interrupts can only be 0 or 1, line {}", i));
                        }
                        parsed_number
                    } else {
                        parse_register(tokens[1], i)?
                    };
                    let opcode: Option<OpCode> = match tokens[0] {
                        "OUT" => Some(OpCode::OUT(number)),
                        "IN" => Some(OpCode::IN(number)),
                        "PUSH" => Some(OpCode::PUSH(number)),
                        "POP" => Some(OpCode::POP(number)),
                        "JZ" => Some(OpCode::JZ(number)),
                        "JN" => Some(OpCode::JN(number)),
                        "JC" => Some(OpCode::JC(number)),
                        "JMP" => Some(OpCode::JMP(number)),
                        "CALL" => Some(OpCode::CALL(number)),
                        "INT" => Some(OpCode::INT(number)),
                        _ => None,
                    };
                    if let Some(opcode) = opcode {
                        code_features.push(CodeFeature::OpCode(opcode));
                        continue;
                    }
                } else if tokens.len() == 3 {
                    if tokens[0] == "LDM" {
                        let rdst = parse_register(tokens[1], i)?;
                        let number = tokens[2].parse::<i16>();
                        if number.is_err() {
                            return Err(format!("Invalid number on line {}", i));
                        }
                        let number = number.unwrap();
                        code_features.push(CodeFeature::OpCode(OpCode::LDM(rdst, number)));
                        continue;
                    } else {
                        let register1 = parse_register(tokens[1], i)?;
                        let register2 = parse_register(tokens[2], i)?;

                        let opcode: Option<OpCode> = match tokens[0] {
                            "NOT" => Some(OpCode::NOT(register1, register2)),
                            "INC" => Some(OpCode::INC(register1, register2)),
                            "MOV" => Some(OpCode::MOV(register1, register2)),
                            _ => None,
                        };
                        if let Some(opcode) = opcode {
                            code_features.push(CodeFeature::OpCode(opcode));
                        }
                    }
                } else if tokens.len() == 4 {
                    let register1 = parse_register(tokens[1], i)?;
                    let register2 = parse_register(tokens[2], i)?;
                    if matches!(tokens[0], "IADD" | "LDD" | "STD") {
                        let Ok(extra_value) = tokens[3].parse::<i16>() else {
                            return Err(format!("Invalid number on line {}", i));
                        };

                        let opcode = match tokens[0] {
                            "IADD" => OpCode::IADD(register1, register2, extra_value),
                            "LDD" => OpCode::LDD(register1, register2, extra_value),
                            "STD" => OpCode::STD(register1, register2, extra_value),
                            _ => unreachable!(),
                        };

                        code_features.push(CodeFeature::OpCode(opcode));
                        continue;
                    } else if matches!(tokens[0], "ADD" | "SUB" | "AND") {
                        let register3 = parse_register(tokens[3], i)?;

                        let opcode = match tokens[0] {
                            "ADD" => OpCode::ADD(register1, register2, register3),
                            "SUB" => OpCode::SUB(register1, register2, register3),
                            "AND" => OpCode::AND(register1, register2, register3),
                            _ => unreachable!(),
                        };

                        code_features.push(CodeFeature::OpCode(opcode));
                        continue;
                    }
                }
                return Err(format!("Invalid instruction or label on line {}", i));
            }
        }
    }

    Ok(code_features)
}

fn parse_register(value: &str, line_number: usize) -> Result<u16, String> {
    let value = value.strip_prefix("R");
    if let Some(value) = value {
        if let Ok(value) = value.parse::<i16>() {
            if (0..8).contains(&value) {
                Ok(value as u16)
            } else {
                Err(format!("Registers are only 0-7, line {}", line_number))
            }
        } else {
            Err(format!("Invalid register syntax, line {}", line_number))
        }
    } else {
        Err(format!(
            "Registers have to start with R, line {}",
            line_number
        ))
    }
}

fn find_opcodes<'a>(tokens: &'a [CodeFeature], label: &'static str) -> &'a [CodeFeature] {
    let handler = tokens.iter().position(|c| {
        if let CodeFeature::Label(clabel) = c {
            *clabel == label
        } else {
            false
        }
    });

    match handler {
        Some(index) => {
            let next_label_index = tokens
                .iter()
                .skip(index + 1)
                .position(|c| matches!(c, CodeFeature::Label(_)))
                .map(|relative_pos| index + 1 + relative_pos);
            match next_label_index {
                Some(next_index) => &tokens[index + 1..next_index],
                None => &tokens[index + 1..],
            }
        }
        None => &[CodeFeature::OpCode(OpCode::HLT)],
    }
}

pub fn parse(lines: &[&str]) -> Result<Vec<u16>, String> {
    let tokens = lex(lines)?;

    let stack_underflow_handler = find_opcodes(&tokens, "Stack_Underflow_Handler");
    let invalid_memory_address_handler = find_opcodes(&tokens, "Invalid_Memory_Address_Handler");
    let interrupt_0 = find_opcodes(&tokens, "Interrupt 0");
    let interrupt_1 = find_opcodes(&tokens, "Interrupt 1");
    let main = find_opcodes(&tokens, "Main");

    let stack_underflow_handler: Vec<OpCode> = stack_underflow_handler
        .iter()
        .map(|c| match c {
            CodeFeature::OpCode(opcode) => *opcode,
            _ => unreachable!(),
        })
        .collect();
    let invalid_memory_address_handler: Vec<OpCode> = invalid_memory_address_handler
        .iter()
        .map(|c| match c {
            CodeFeature::OpCode(opcode) => *opcode,
            _ => unreachable!(),
        })
        .collect();
    let interrupt_0: Vec<OpCode> = interrupt_0
        .iter()
        .map(|c| match c {
            CodeFeature::OpCode(opcode) => *opcode,
            _ => unreachable!(),
        })
        .collect();
    let interrupt_1: Vec<OpCode> = interrupt_1
        .iter()
        .map(|c| match c {
            CodeFeature::OpCode(opcode) => *opcode,
            _ => unreachable!(),
        })
        .collect();
    let main: Vec<OpCode> = main
        .iter()
        .map(|c| match c {
            CodeFeature::OpCode(opcode) => *opcode,
            _ => unreachable!(),
        })
        .collect();

    let stack_underflow_handler = encode_opcodes(&stack_underflow_handler);
    let invalid_memory_address_handler = encode_opcodes(&invalid_memory_address_handler);
    let interrupt_0 = encode_opcodes(&interrupt_0);
    let interrupt_1 = encode_opcodes(&interrupt_1);
    let main = encode_opcodes(&main);

    let code_length = stack_underflow_handler.len()
        + invalid_memory_address_handler.len()
        + interrupt_0.len()
        + interrupt_1.len()
        + main.len();

    if code_length > 4090 {
        return Err("Program is too big".to_string());
    }
    let mut bytes: Vec<u16> = vec![0; code_length + 6];

    let invalid_memory_address_handler_address = 4 + stack_underflow_handler.len();
    let interrupt_table_address =
        invalid_memory_address_handler_address + invalid_memory_address_handler.len();
    let interrupt_0_address = interrupt_table_address + 2;
    let interrupt_1_address = interrupt_0_address + interrupt_0.len();
    let main_address = interrupt_1_address + interrupt_1.len();

    println!(
        "Main: {}\nUnderflow handler: {}\nInvalid memory address handler: {}\nInterrupt 0: {}\nInterrupt 1: {}",
        main_address,
        4,
        invalid_memory_address_handler_address,
        interrupt_0_address,
        interrupt_1_address
    );

    bytes[0] = main_address as u16;
    bytes[1] = 4;
    bytes[2] = invalid_memory_address_handler_address as u16;
    bytes[3] = (4 + stack_underflow_handler.len() + invalid_memory_address_handler.len()) as u16;
    bytes[4..4 + stack_underflow_handler.len()].copy_from_slice(&stack_underflow_handler);
    bytes[invalid_memory_address_handler_address
        ..invalid_memory_address_handler_address + invalid_memory_address_handler.len()]
        .copy_from_slice(&invalid_memory_address_handler);
    bytes[interrupt_table_address] = interrupt_0_address as u16;
    bytes[interrupt_table_address + 1] = interrupt_1_address as u16;
    bytes[interrupt_0_address..interrupt_0_address + interrupt_0.len()]
        .copy_from_slice(&interrupt_0);
    bytes[interrupt_1_address..interrupt_1_address + interrupt_1.len()]
        .copy_from_slice(&interrupt_1);
    bytes[main_address..main_address + main.len()].copy_from_slice(&main);

    Ok(bytes)
}
