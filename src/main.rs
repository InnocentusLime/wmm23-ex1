use machine_memory::{Threads, Memory, MemorySubsystem, AccessMode};
use machine_thread::CodeInstruction;
use value::Value;

mod machine_thread;
mod machine_memory;
mod value;
mod register;
mod label;

#[derive(Debug, Clone, Copy)]
pub enum MachineEvent {
    Silent,
    Read {
        tid: usize,
        location: usize,
        value: Value,
        mode: AccessMode,
    },
    Write {
        tid: usize,
        location: usize,
        value: Value,
        mode: AccessMode,
    },
    Fence {
        tid: usize,
        mode: AccessMode,
    },
    Rmw {
        tid: usize,
        location: usize,
        read_value: Value,
        write_value: Value,
        mode: AccessMode,
    }
}

pub enum MachineError {

}

#[derive(Debug)]
pub enum MachineStep<Mem: MemorySubsystem> {
    Thread(usize),
    Memory(Mem::Independent),
}

pub struct Machine<'a, Mem> {
    threads: Threads<'a, Mem>,
    memory: Memory<Mem>,
}

impl<'a, Mem: MemorySubsystem> Machine<'a, Mem> {
    pub fn new(program: &'a [Vec<CodeInstruction>]) -> Result<Machine<'a, Mem>, MachineError> {
        todo!()
    }

    pub fn step(&mut self, step: MachineStep<Mem>) -> Result<MachineEvent, MachineError> {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}
