use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Value(pub u64);

impl Value {
    pub fn to_address(self) -> usize { self.0 as usize }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self { Self(value) }
}