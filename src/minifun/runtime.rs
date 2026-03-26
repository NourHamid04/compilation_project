use std::collections::HashMap;

use crate::minifun::ast::Term;

// Values produced by MiniFun evaluation.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),

    // Non-recursive closure: parameter, body, saved environment
    Closure(String, Box<Term>, Env),

    // Recursive closure: function name, parameter, body, saved environment
    RecClosure(String, String, Box<Term>, Env),
}

/// MiniFun environment:
/// maps variable names to values.
pub type Env = HashMap<String, Value>;