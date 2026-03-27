use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Fun(Box<Type>, Box<Type>),
}

pub type TypeEnv = HashMap<String, Type>;