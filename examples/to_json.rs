// Copyright 2017 6WIND S.A. <quentin.monnet@6wind.com>
//
// Licensed under the Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> or
// the MIT license <http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[macro_use]
extern crate json;

extern crate elf;
use std::path::PathBuf;

extern crate gemachain_rbpf;
use gemachain_rbpf::{
    disassembler::disassemble_instruction,
    static_analysis::Analysis,
    user_error::UserError,
    vm::{Config, Executable, SyscallRegistry, TestInstructionMeter},
};
use std::collections::BTreeMap;
// Turn a program into a JSON string.
//
// Relies on `json` crate.
//
// You may copy this function and adapt it according to your needs. For instance, you may want to:
//
// * Remove the "desc" (description) attributes from the output.
// * Print integers as integers, and not as strings containing their hexadecimal representation
//   (just replace the relevant `format!()` calls by the commented values.
fn to_json(program: &[u8]) -> String {
    let executable = <dyn Executable<UserError, TestInstructionMeter>>::from_text_bytes(
        &program,
        None,
        Config::default(),
        SyscallRegistry::default(),
        BTreeMap::default(),
    )
    .unwrap();
    let analysis = Analysis::from_executable(executable.as_ref());

    let mut json_insns = vec![];
    for insn in analysis.instructions.iter() {
        json_insns.push(object!(
            "opc"  => format!("{:#x}", insn.opc), // => insn.opc,
            "dst"  => format!("{:#x}", insn.dst), // => insn.dst,
            "src"  => format!("{:#x}", insn.src), // => insn.src,
            "off"  => format!("{:#x}", insn.off), // => insn.off,
            // Warning: for imm we use a i64 instead of a i32 (to have correct values for
            // `lddw` operation. If we print a number in the JSON this is not a problem, the
            // internal i64 has the same value with extended sign on 32 most significant bytes.
            // If we print the hexadecimal value as a string however, we want to cast as a i32
            // to prevent all other instructions to print spurious `ffffffff` prefix if the
            // number is negative. When values takes more than 32 bits with `lddw`, the cast
            // has no effect and the complete value is printed anyway.
            "imm"  => format!("{:#x}", insn.imm as i32), // => insn.imm,
            "desc" => disassemble_instruction(&insn, &analysis),
        ));
    }
    json::stringify_pretty(
        object!(
        "size"  => json_insns.len(),
        "insns" => json_insns
        ),
        4,
    )
}

// Load a program from an object file, and prints it to standard output as a JSON string.
fn main() {
    // Let's reuse this file from `load_elf` example.
    let filename = "examples/load_elf__block_a_port.o";

    let path = PathBuf::from(filename);
    let file = match elf::File::open_path(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error: {:?}", e),
    };

    let text_scn = match file.get_section(".classifier") {
        Some(s) => s,
        None => panic!("Failed to look up .classifier section"),
    };

    let prog = &text_scn.data;

    println!("{}", to_json(prog));
}
