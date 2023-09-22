
use thiserror::Error;
use tracing::debug;

use super::{MemorySubsystem, MemoryStep, Threads, GlobalMemory, MemoryError, MemoryQuery};

#[derive(Debug)]
pub enum IndependentStep {}

#[derive(Debug, Error)]
pub enum Error {}

pub struct ScMemory;

impl ScMemory {
    fn serve_thread_request(
        &mut self,
        tid: usize,
        query: MemoryQuery,
        threads: &mut Threads<Self>,
        memory: &mut GlobalMemory<Self>,
    ) -> Result<(), MemoryError<Error>> {
        let thread_state = threads.get_thread_mut(tid)?;
        match query {
            super::MemoryQuery::Store {
                addr,
                value,
                ..
            } => {
                let target = memory.fetch(addr)?;
                *target = value;

                Ok(())
            },
            super::MemoryQuery::Load {
                addr,
                dest,
                ..
            } => {
                let val = memory.fetch(addr)?;
                thread_state.set_register(dest, *val)
                    .map_err(|error| MemoryError::ThreadStateError { tid, error })?;

                Ok(())
            },
            super::MemoryQuery::Cas {
                addr,
                expected,
                new_value,
                ..
            } => {
                let val = memory.fetch(addr)?;
                if expected != *val {
                    debug!("CAS fail");
                    return Ok(());
                }

                *val = new_value;
                Ok(())
            },
            super::MemoryQuery::Fai {
                addr,
                dest,
                ..
            } => {
                let val = memory.fetch(addr)?;
                thread_state.set_register(dest, *val)
                    .map_err(|error| MemoryError::ThreadStateError { tid, error })?;

                // TODO probbaly shouldn't peek into "val" internals
                val.0 += 1;

                Ok(())
            },
            super::MemoryQuery::Fence { .. } => Ok(()),
        }
    }
}

impl MemorySubsystem for ScMemory {
    type Err = Error;
    type Independent = IndependentStep;

    fn name() -> &'static str { "SC" }

    fn execute_step(
        &mut self,
        step: MemoryStep<Self::Independent>,
        threads: &mut Threads<Self>,
        memory: &mut GlobalMemory<Self>,
    ) -> Result<(), MemoryError<Self::Err>> {
        debug!("Step: {step:?}");

        match step {
            MemoryStep::Independent(x) => match x {},
            MemoryStep::ThreadRequest { tid, query } => self.serve_thread_request(
                tid,
                query,
                threads,
                memory,
            ),
        }
    }
}