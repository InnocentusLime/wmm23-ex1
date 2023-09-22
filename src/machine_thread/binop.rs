use std::fmt;

use crate::value::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BinOpError {
    #[error("Operation {op:?} with operands {l:?} and {r:?} has overflown")]
    Overflow {
        l: Value,
        r: Value,
        op: BinOp,
    },
    #[error("Operation {op:?} with operands {l:?} and {r:?} has underflown")]
    Underflow {
        l: Value,
        r: Value,
        op: BinOp,
    },
    #[error("A division by zero has occured")]
    DivisionByZero,
}

/// Binary operations supported by the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    /// Addition.
    Add,
    /// Substraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
        }
    }
}

impl BinOp {
    pub fn eval(self, l: Value, r: Value) -> Result<Value, BinOpError> {
        let op = self;

        match op {
            BinOp::Add => l.0.checked_add(r.0).ok_or(BinOpError::Overflow { l, r, op }),
            BinOp::Sub => l.0.checked_sub(r.0).ok_or(BinOpError::Underflow { l, r, op }),
            BinOp::Mul => l.0.checked_mul(r.0).ok_or(BinOpError::Overflow { l, r, op }),
            BinOp::Div => l.0.checked_div(r.0).ok_or(BinOpError::DivisionByZero),
        }.map(Value)
    }
}