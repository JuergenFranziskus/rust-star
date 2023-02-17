use std::fmt::Display;

use super::types::Type;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Register {
    id: RegisterID,
    register_type: Type,
}
impl Register {
    pub fn new(id: RegisterID, register_type: Type) -> Self {
        Self { id, register_type }
    }

    pub fn register_type(&self) -> Type {
        self.register_type
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RegisterID(pub(super) usize);
impl From<usize> for RegisterID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for RegisterID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}
