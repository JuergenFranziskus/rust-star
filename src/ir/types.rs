use crate::util::arena::{Arena, ID};

pub struct Types {
    functions: Arena<FunctionType>,
    arrays: Arena<ArrayType>,
}
impl Types {
    pub fn new() -> Types {
        Types {
            functions: Arena::new(),
            arrays: Arena::new(),
        }
    }

    pub fn make_function(&mut self, ret: Type, params: Vec<Type>) -> ID<FunctionType> {
        if let Some(id) = self
            .functions
            .find(|t| t.return_type == ret && t.parameters == params)
        {
            id
        } else {
            let id = self.functions.next_id();
            self.functions.push(FunctionType {
                id,
                return_type: ret,
                parameters: params,
            })
        }
    }
    pub fn make_array(&mut self, member: Type, length: usize) -> ID<ArrayType> {
        if let Some(id) = self
            .arrays
            .find(|t| t.member_type == member && t.length == length)
        {
            id
        } else {
            let id = self.arrays.next_id();
            self.arrays.push(ArrayType {
                id,
                member_type: member,
                length,
            })
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    I1,
    I8,
    I16,
    I32,
    I64,

    Pointer,
    Function(ID<FunctionType>),
    Array(ID<ArrayType>),
}
impl From<ID<FunctionType>> for Type {
    fn from(value: ID<FunctionType>) -> Self {
        Self::Function(value)
    }
}
impl From<ID<ArrayType>> for Type {
    fn from(value: ID<ArrayType>) -> Self {
        Self::Array(value)
    }
}

pub struct FunctionType {
    id: ID<FunctionType>,
    return_type: Type,
    parameters: Vec<Type>,
}
impl FunctionType {
    pub fn id(&self) -> ID<FunctionType> {
        self.id
    }
    pub fn return_type(&self) -> Type {
        self.return_type
    }
    pub fn parameters(&self) -> &[Type] {
        &self.parameters
    }
}

pub struct ArrayType {
    id: ID<ArrayType>,
    member_type: Type,
    length: usize,
}
impl ArrayType {
    pub fn id(&self) -> ID<ArrayType> {
        self.id
    }
    pub fn member_type(&self) -> Type {
        self.member_type
    }
    pub fn length(&self) -> usize {
        self.length
    }
}
