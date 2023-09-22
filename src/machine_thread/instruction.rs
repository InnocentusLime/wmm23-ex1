use std::fmt;

use crate::{value::Value, machine_memory::{AccessMode, MemoryQuery}, register::RegisterRef};

use super::{Register, binop::BinOp, Label, ThreadState, ThreadStateError};

use smallvec::{SmallVec, smallvec};

/// All the instructions of the simple machine.
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Stores [`value`](Instruction::Set::value) in [`dest`](Instruction::Set::value) register.
    ///
    /// # Semantics
    /// ```
    /// R[dest] = value
    /// ```
    Set { dest: Register, value: Value },
    /// Stores the result of applying [`op`](Instruction::Bop::dest) to
    /// [`src_l`](Instruction::Bop::src_l) and [`src_r`](Instruction::Bop::src_r)
    /// registers into [`dest`](Instruction::Bop::dest) reigster.
    ///
    /// # Semantics
    /// ```
    /// R[dest] = R[src_l] `op` R[src_r]
    /// ```
    Bop { dest: Register, binop: BinOp, src_l: Register, src_r: Register },
    /// Performs a conditional jump to [`label`](Instruction::Branch::label) if
    /// [`src`](Instruction::Branch::src) register contains a non-zero value.
    ///
    /// # Semantics
    /// ```
    /// if(R[src] != 0) PC = label
    /// ```
    Branch { src: Register, label: Label },
    /// Loads a value from address specified by [`addr`](Instruction::Load::addr) register into
    /// [`dest`](Instruction::Load::dest) register with access mode
    /// set to [`mode`](Instruction::Load::mode).
    ///
    /// # Semantics
    /// ```
    /// R[dest] = M[R[addr]] with `mode`
    /// ```
    Load { mode: AccessMode, addr: Register, dest: Register },
    /// Set the value stored at address specified by [`addr`](Instruction::Load::addr) register
    /// to [`src`](Instruction::Load::dest) with access mode
    /// set to [`mode`](Instruction::Load::mode).
    ///
    /// # Semantics
    /// ```
    /// M[R[addr]] = R[src] with `mode`
    /// ```
    Store { mode: AccessMode, addr: Register, src: Register },
    /// Performs a compare-and-swap operation on the value stored at address specified by
    /// [`addr`](Instruction::Load::addr) register. The expected value is specified by
    /// [`expected`](Instruction::Cas::expected) register and the new value is specified by
    /// the [`new_value`](Instruction::Cas::new_value) register.
    ///
    /// # Semantics
    /// ```
    /// if (M[R[addr]] == R[expected]) M[R[addr]] = R[src] with `mode`
    /// ```
    Cas { mode: AccessMode, addr: Register, expected: Register, new_value: Register },
    /// Performs a fetch-and-increment operation on the value stored at address specified by
    /// [`addr`](Instruction::Fai::addr) register. The old value will be loaded into the
    /// [`dest`](Instruction::Fai::dest) register with access mode
    /// set to [`mode`](Instruction::Load::mode).
    ///
    /// # Semantics
    /// ```
    /// R[dest] = M[R[addr]] then immediate M[R[addr]] = M[R[addr]] + 1 with `mode`
    /// ```
    Fai { mode: AccessMode, addr: Register, dest: Register },
    /// A memory fence with access mode set to [`mode`](Instruction::Fence::mode).
    Fence { mode: AccessMode },
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Set { dest, value } => write!(f, "{dest} = {value}"),
            Instruction::Bop {
                dest,
                binop: op,
                src_l,
                src_r,
            } => write!(f, "{dest} = {src_l} {op} {src_r}"),
            Instruction::Branch { src, label } => write!(f, "if {src} goto {label}"),
            Instruction::Load {
                mode,
                addr,
                dest,
            } => write!(f, "load {mode} ##{addr} {dest}"),
            Instruction::Store {
                mode,
                addr,
                src ,
            } => write!(f, "store {mode} ##{addr} {src}"),
            Instruction::Cas {
                mode,
                addr,
                expected,
                new_value: src,
            } => write!(f, "cas {mode} ##{addr} {expected} {src}"),
            Instruction::Fai {
                mode,
                addr,
                dest,
            } => write!(f, "fai {mode} ##{addr} {dest}"),
            Instruction::Fence { mode } => write!(f, "fence {mode}"),
        }
    }
}

impl Instruction {
    // TODO returns yarn-refs. They can then be immortalised
    pub(super) fn used_registers(&self) -> SmallVec<[RegisterRef; 3]> {
        match self {
            Instruction::Set { dest, .. } => smallvec![
                dest.as_ref()
            ],
            Instruction::Bop {
                dest,
                src_l,
                src_r,
                ..
            } => smallvec![dest.as_ref(), src_l.as_ref(), src_r.as_ref()],
            Instruction::Branch { src, .. } => smallvec![src.as_ref()],
            Instruction::Load {
                addr,
                dest,
                ..
            } => smallvec![addr.as_ref(), dest.as_ref()],
            Instruction::Store {
                addr,
                src,
                ..
            } => smallvec![addr.as_ref(), src.as_ref()],
            Instruction::Cas {
                addr,
                expected,
                new_value: src,
                ..
            } => smallvec![addr.as_ref(), expected.as_ref(), src.as_ref()],
            Instruction::Fai {
                addr,
                dest,
                ..
            } => smallvec![addr.as_ref(), dest.as_ref()],
            Instruction::Fence { .. } => smallvec![],
        }
    }

    pub(super) fn execute(&self, state: &mut ThreadState) -> Result<Option<MemoryQuery>, ThreadStateError> {
        match self {
            Instruction::Set { dest, value } => {
                state.set_register(dest.as_ref(),*value);

                Ok(None)
            },
            Instruction::Bop { dest, binop, src_l, src_r } => {
                let val_l = state.get_register(src_l.as_ref())?;
                let val_r = state.get_register(src_r.as_ref())?;
                let val = binop.eval(val_l, val_r)
                    .map_err(|err| ThreadStateError::BinOpError {
                        binop: *binop,
                        err
                    })?;

                state.set_register(dest.as_ref(), val)?;

                Ok(None)
            },
            Instruction::Branch { src, label } => {
                let val = state.get_register(src.as_ref())?;
                if val.0 != 0 {
                    state.goto_label(label.as_ref())?;
                }

                Ok(None)
            },
            Instruction::Load { mode, addr, dest } => {
                let addr = state.get_register(addr.as_ref())?.to_address();

                Ok(Some(MemoryQuery::Load {
                    addr,
                    dest: dest.as_ref(),
                    mode: *mode
                }))
            },
            Instruction::Store { mode, addr, src } => {
                let addr = state.get_register(addr.as_ref())?.to_address();
                let value = state.get_register(src.as_ref())?;

                Ok(Some(MemoryQuery::Store {
                    addr,
                    value,
                    mode: *mode
                }))
            },
            Instruction::Cas { mode, addr, expected, new_value } => {
                let addr = state.get_register(addr.as_ref())?.to_address();
                let expected = state.get_register(expected.as_ref())?;
                let new_value = state.get_register(new_value.as_ref())?;

                Ok(Some(MemoryQuery::Cas {
                    addr,
                    expected,
                    new_value,
                    mode: *mode,
                }))
            },
            Instruction::Fai { mode, addr, dest } => {
                let addr = state.get_register(addr.as_ref())?.to_address();

                Ok(Some(MemoryQuery::Fai {
                    addr,
                    dest: dest.as_ref(),
                    mode: *mode
                }))
            },
            Instruction::Fence { mode } => Ok(Some(
                MemoryQuery::Fence { mode: *mode }
            )),
        }
    }
}