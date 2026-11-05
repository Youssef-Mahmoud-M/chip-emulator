use std::fs::File;
use std::io::Write;
use std::{env, fs};

use computer_architecture::{ChipState, assembler};

fn main() {
    let args: Vec<String> = env::args_os()
        .map(|s| s.to_string_lossy().to_string())
        .skip(1)
        .collect();
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    match args[..] {
        ["assemble", file, output] => {
            let Ok(exists) = fs::exists(file) else {
                eprintln!("Failed to check assembly file");
                return;
            };
            if !exists {
                eprintln!("Please pass a valid assembly file");
                return;
            }
            let Ok(contents) = fs::read_to_string(file) else {
                eprintln!("Could not read file");
                return;
            };

            let contents: Vec<&str> = contents.split("\n").collect();

            let assembly = match assembler::parse(&contents) {
                Ok(assembly) => assembly,
                Err(error) => {
                    eprintln!("{}", error);
                    return;
                }
            };

            let Ok(mut file) = File::create(output) else {
                eprintln!("Failed to create output file");
                return;
            };

            let byte_slice: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    assembly.as_ptr() as *const u8,
                    assembly.len() * std::mem::size_of::<u16>(),
                )
            };

            if file.write_all(byte_slice).is_err() {
                eprintln!("Failed to write to output file");
            }
        }
        ["run", file, port_in] => {
            let Ok(port_in) = port_in.parse::<i16>() else {
                eprintln!("Pass a valid number to portIn");
                return;
            };

            let Ok(exists) = fs::exists(file) else {
                eprintln!("Failed to check assembly file");
                return;
            };
            if !exists {
                eprintln!("Please pass a valid assembly file");
                return;
            }

            let Ok(contents) = fs::read(file) else {
                eprintln!("Failed to read file");
                return;
            };

            if contents.len() % 2 != 0 {
                eprintln!("File is not a valid binary");
                return;
            }

            let words: Vec<u16> = contents
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();

            if words.len() > 4096 {
                eprintln!("File is too large");
                return;
            }

            let mut bytes: [u16; 4096] = [0; 4096];
            bytes[0..words.len()].copy_from_slice(&words);

            let mut state: ChipState = ChipState::new(bytes, port_in);
            let mut status = true;
            while status {
                status = state.cycle();
            }
        }
        _ => {
            eprintln!("Please pass a valid set of parameters")
        }
    }
}
