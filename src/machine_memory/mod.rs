mod sc;

use std::{fmt::{self, Debug}, error::Error, marker::PhantomData};
use thiserror::Error;

use crate::{value::Value, register::RegisterRef, machine_thread::{ThreadState, ThreadStateError}};

/// Memory access mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AccessMode {
    /// Sequential Consistency
    SeqCst,
    /// Release
    Rel,
    // Acquire
    Acq,
    // Release-Acquire
    RelAcq,
    // Relaxed
    Rlx,
}

impl fmt::Display for AccessMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessMode::SeqCst => write!(f, "SEQ_CST"),
            AccessMode::Rel => write!(f, "REL"),
            AccessMode::Acq => write!(f, "ACQ"),
            AccessMode::RelAcq => write!(f, "REL_ACQ"),
            AccessMode::Rlx => write!(f, "RLX"),
        }
    }
}

/// The query for the memory subsystem.
#[derive(Clone, Copy, Debug)]
pub enum MemoryQuery<'a> {
    /// A query to write some value.
    Store {
        addr: usize,
        value: Value,
        mode: AccessMode,
    },
    /// A query to read a value into a register.
    Load {
        addr: usize,
        dest: RegisterRef<'a>,
        mode: AccessMode,
    },
    /// Compare-and-swap
    Cas {
        addr: usize,
        expected: Value,
        new_value: Value,
        mode: AccessMode,
    },
    /// Fetch-and-increment
    Fai {
        addr: usize,
        dest: RegisterRef<'a>,
        mode: AccessMode,
    },
    /// Instructing the memory subsystem to perform a fence.
    Fence {
        mode: AccessMode,
    },
}

#[derive(Debug)]
pub enum MemoryStep<'a, S> {
    Independent(S),
    ThreadRequest {
        tid: usize,
        query: MemoryQuery<'a>,
    },
}

// TODO make into a global error
// TODO remove generics in favour of `anyhow`
#[derive(Debug, Error)]
pub enum MemoryError<E> {
    #[error("Address {addr} out of range")]
    AddressOutOfRange {
        addr: usize,
    },
    #[error("Thread ID {tid} is incorrect")]
    BadTid {
        tid: usize,
    },
    #[error("Call to thread state {tid} API has returned an error")]
    ThreadStateError {
        tid: usize,
        error: ThreadStateError,
    },
    #[error("Memory system failed with implementation specific error")]
    Other(#[from] E),
}

pub struct GlobalMemory<Mem> {
    mem: Vec<Value>,
    _phantom: PhantomData<fn(&Mem) -> ()>,
}

impl<Mem: MemorySubsystem> GlobalMemory<Mem> {
    pub fn fetch(&mut self, addr: usize) -> Result<&mut Value, MemoryError<Mem::Err>> {
        match self.mem.get_mut(addr) {
            Some(x) => Ok(x),
            None => Err(MemoryError::AddressOutOfRange { addr }),
        }
    }
}

// TODO probably should move out
pub struct Threads<'prog, Mem> {
    threads: Vec<ThreadState<'prog>>,
    _phantom: PhantomData<fn(&Mem) -> ()>,
}

impl<'prog, Mem: MemorySubsystem> Threads<'prog, Mem> {
    pub fn get_thread_mut(&mut self, tid: usize) -> Result<&mut ThreadState<'prog>, MemoryError<Mem::Err>> {
        match self.threads.get_mut(tid) {
            Some(x) => Ok(x),
            None => Err(MemoryError::BadTid { tid }),
        }
    }
}

pub trait MemorySubsystem: Sized {
    type Err: Error;
    type Independent: Debug;

    fn name() -> &'static str;
    fn execute_step(
        &mut self,
        step: MemoryStep<Self::Independent>,
        threads: &mut Threads<Self>,
        memory: &mut GlobalMemory<Self>,
    ) -> Result<(), MemoryError<Self::Err>>;
}

pub struct Memory<T> {
    subsystem: T,
    global: GlobalMemory<T>,
}