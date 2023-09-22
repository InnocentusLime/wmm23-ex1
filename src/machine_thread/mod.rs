
mod binop;
mod instruction;

use tracing::{ debug, trace };
use fnv::FnvHashMap;
use instruction::Instruction;
use thiserror::Error;

use crate::{value::Value, register::{Register, RegisterRef}, label::{Label, LabelRef}, machine_memory::MemoryQuery};

use self::binop::{BinOpError, BinOp};

#[derive(Debug, Clone)]
pub struct CodeInstruction {
    pub label: Option<Label>,
    pub instruction: Instruction,
}

#[derive(Debug)]
pub struct ThreadState<'a> {
    reg_map: FnvHashMap<Register, Value>,
    label_map: FnvHashMap<Label, usize>,
    program: &'a [CodeInstruction],
    pc: usize,
}

#[derive(Debug, Error)]
pub enum ThreadStateCreationError {
    #[error("Received an empty program")]
    EmptyProgram,
    #[error("Duplicate label \"{label}\" at instruction {first} and {second}")]
    DuplicateLabel {
        label: Label,
        first: usize,
        second: usize,
    },
}

#[derive(Debug, Error)]
pub enum ThreadStateError {
    #[error("Register {register} is not used in this program")]
    UnboundRegister {
        register: Register,
    },
    #[error("Label {label} is not used anywhere in this program")]
    UnboundLabel {
        label: Label,
    },
    #[error("PC value ({address}) out of range")]
    PcOutOfRange {
        address: usize,
    },
    #[error("Binop \"{binop}\" has failed to execute")]
    BinOpError {
        binop: BinOp,
        #[source] err: BinOpError,
    },
}

impl<'a> ThreadState<'a> {
    fn add_label(label_map: &mut FnvHashMap<Label, usize>, label: Label, addr: usize) -> Result<(), ThreadStateCreationError> {

        if let Some(old_addr) = label_map.insert(label.clone(), addr) {
            return Err(ThreadStateCreationError::DuplicateLabel {
                label,
                first: addr,
                second: old_addr
            });
        }

        debug!("Added \"{label}\" for {addr}");

        Ok(())
    }

    pub fn new(program: &'a [CodeInstruction]) -> Result<Self, ThreadStateCreationError> {
        if program.is_empty() {
            return Err(ThreadStateCreationError::EmptyProgram);
        }

        let mut reg_map = FnvHashMap::default();
        let mut label_map = FnvHashMap::default();

        for (addr, code_instruction) in program.iter().enumerate() {
            let used_registers = code_instruction.instruction.used_registers();

            debug!("{addr:0>5}\t{:>32}\t{:?}", code_instruction.instruction, used_registers);
            used_registers.into_iter().for_each(|x| {
                reg_map.insert(x.to_box().immortalize(), Value(0));
            });

            if let Some(label) = &code_instruction.label {
                Self::add_label(&mut label_map, label.to_owned(), addr)?;
            }
        }

        Ok(ThreadState { reg_map, label_map, program, pc: 0 })
    }

    pub fn set_register(&mut self, register: RegisterRef, val: Value) -> Result<(), ThreadStateError> {
        debug!("{register} <- {val}");

        match self.reg_map.get_mut(register.as_str()) {
            Some(x) => { *x = val; Ok(()) },
            None => Err(ThreadStateError::UnboundRegister {
                register: register.to_box().immortalize(),
            }),
        }
    }

    pub fn get_register(&self, register: RegisterRef) -> Result<Value, ThreadStateError> {
        debug!("{register} ->");

        match self.reg_map.get(&register.to_box()) {
            Some(x) => Ok(*x),
            None => Err(ThreadStateError::UnboundRegister {
                register: register.to_box().immortalize(),
            }),
        }
    }

    pub fn goto_label(&mut self, label: LabelRef) -> Result<(), ThreadStateError> {
        debug!("GOTO {label:?}");

        match self.label_map.get(&label.to_box()) {
            Some(x) => { self.pc = *x; Ok(()) },
            None => Err(ThreadStateError::UnboundLabel {
                label: label.to_box().immortalize(),
            }),
        }
    }

    pub fn step(&mut self) -> Result<Option<MemoryQuery<'a>>, ThreadStateError> {
        let instruction_to_run = self.pc;
        let instruction = self.program.get(instruction_to_run)
            .ok_or(ThreadStateError::PcOutOfRange { address: self.pc })?;

        self.pc += 1;
        let res = instruction.instruction.execute(self);
        trace!(
            "{instruction_to_run:0>5} -> {:0>5} {:>32}: {res:?}",
            self.pc,
            instruction.instruction
        );

        res
    }
}