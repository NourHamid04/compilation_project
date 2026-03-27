use crate::minifun::types::Type;

#[derive(Debug, Clone)]
pub enum Term {
    Int(i64),
    True,
    False,
    Var(String),

    Fun(String, Type, Box<Term>),
    App(Box<Term>, Box<Term>),

    Add(Box<Term>, Box<Term>),
    Sub(Box<Term>, Box<Term>),
    Mul(Box<Term>, Box<Term>),
    And(Box<Term>, Box<Term>),
    Less(Box<Term>, Box<Term>),
    Not(Box<Term>),

    If(Box<Term>, Box<Term>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),

    LetFun(String, String, Type, Box<Term>, Box<Term>),
}