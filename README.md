A 16-bit chip emulator and assembler.

## Instruction format

Every instruction is one 16-bit word. Instructions that take an immediate or offset are followed by a second word holding that value.

| Bits | 15–9 | 8–6 | 5–3 | 2–0 |
|---|---|---|---|---|
| Field | Opcode | Rdst | Rsrc1 | Rsrc2 |

Unused fields are 0. The second word (when present) is a 16-bit two's complement value.

## Instructions

| Opcode | Assembly | Words | Fields used | Operation | Flags |
|---|---|---|---|---|---|
| 0 | `NOP` | 1 | — | Do nothing | — |
| 1 | `HLT` | 1 | — | Freeze PC until reset | — |
| 2 | `SETC` | 1 | — | C ← 1 | C |
| 3 | `NOT Rdst Rsrc1` | 1 | Rdst, Rsrc1 | R[Rdst] ← ~R[Rsrc1] | Z, N |
| 4 | `INC Rdst Rsrc1` | 1 | Rdst, Rsrc1 | R[Rdst] ← R[Rsrc1] + 1 | Z, N, C |
| 5 | `OUT Rsrc1` | 1 | Rsrc1 | OUT.PORT ← R[Rsrc1] | — |
| 6 | `IN Rdst` | 1 | Rdst | R[Rdst] ← IN.PORT | — |
| 7 | `MOV Rdst Rsrc1` | 1 | Rdst, Rsrc1 | R[Rdst] ← R[Rsrc1] | — |
| 8 | `ADD Rdst Rsrc1 Rsrc2` | 1 | Rdst, Rsrc1, Rsrc2 | R[Rdst] ← R[Rsrc1] + R[Rsrc2] | Z, N, C |
| 9 | `SUB Rdst Rsrc1 Rsrc2` | 1 | Rdst, Rsrc1, Rsrc2 | R[Rdst] ← R[Rsrc1] − R[Rsrc2] | Z, N, C (borrow) |
| 10 | `AND Rdst Rsrc1 Rsrc2` | 1 | Rdst, Rsrc1, Rsrc2 | R[Rdst] ← R[Rsrc1] AND R[Rsrc2] | Z, N |
| 11 | `PUSH Rsrc1` | 1 | Rsrc1 | DM[SP] ← R[Rsrc1]; SP ← SP − 1 | — |
| 12 | `POP Rdst` | 1 | Rdst | SP ← SP + 1; R[Rdst] ← DM[SP] | — |
| 13 | `JZ Rsrc1` | 1 | Rsrc1 | If Z = 1: PC ← R[Rsrc1] | Clears Z when taken |
| 14 | `JN Rsrc1` | 1 | Rsrc1 | If N = 1: PC ← R[Rsrc1] | Clears N when taken |
| 15 | `JC Rsrc1` | 1 | Rsrc1 | If C = 1: PC ← R[Rsrc1] | Clears C when taken |
| 16 | `JMP Rsrc1` | 1 | Rsrc1 | PC ← R[Rsrc1] | — |
| 17 | `CALL Rsrc1` | 1 | Rsrc1 | DM[SP] ← PC + 1; SP ← SP − 1; PC ← R[Rsrc1] | — |
| 18 | `RET` | 1 | — | SP ← SP + 1; PC ← DM[SP] | — |
| 19 | `INT index` | 1 | Rsrc2 (bit 0 = index, 0 or 1) | DM[SP] ← PC + 1; SP ← SP − 1; save flags; PC ← IM[IM[3] + index] | Saved |
| 20 | `RTI` | 1 | — | SP ← SP + 1; PC ← DM[SP]; restore flags | Restored |
| 21 | `IADD Rdst Rsrc1 Imm` | 2 | Rdst, Rsrc1 | R[Rdst] ← R[Rsrc1] + Imm | Z, N, C |
| 22 | `LDM Rdst Imm` | 2 | Rdst | R[Rdst] ← Imm | — |
| 23 | `LDD Rdst Rsrc1 Offset` | 2 | Rdst, Rsrc1 | R[Rdst] ← DM[R[Rsrc1] + Offset] | — |
| 24 | `STD Rsrc1 Rsrc2 Offset` | 2 | Rsrc1, Rsrc2 | DM[R[Rsrc2] + Offset] ← R[Rsrc1] | — |

Opcodes 25–127 are not defined and execute as `NOP`.

## Usage

Requires Rust 1.88 or newer.

### Assemble

```sh
cargo run --release -- assemble <input.asm> <output.bin>
```

Assembles `input.asm` and writes the program to `output.bin` as 16-bit little-endian words, starting at address 0. The start address of each section is printed so you can work out jump targets.

### Run

```sh
cargo run --release -- run <program.bin> <port_in>
```

Loads `program.bin` into instruction memory, sets `IN.PORT` to `port_in` (a decimal number from -32768 to 32767), and runs from the address in IM[0].

Each `OUT` prints the value written to the output port, and each exception prints a message. The program runs until it reaches `HLT`; a program without `HLT` runs forever, so stop it with Ctrl+C.

### Example

```sh
cargo run --release -- assemble test.asm test.bin
cargo run --release -- run test.bin 5
```

The example multiplies the input by 3
