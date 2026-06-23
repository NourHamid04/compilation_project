use std::fmt;

/// This enum contains all evaluation/runtime errors that may happen while interpreting a MiniImp or MiniFun program.

#[derive(Debug, Clone)]
pub enum EvalError {
    UndefinedVariable(String),
    OutputVariableUndefined(String),
    ParseError(String),
    UnexpectedEndOfInput,
    TypeError(String),
    TypeCheckError(String),

}


impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UndefinedVariable(name) => {
                write!(
                    f,
                    "Runtime error: the variable '{}' was used before it was assigned a value.",
                    name
                )
            }

            EvalError::OutputVariableUndefined(name) => {
                write!(
                    f,
                    "Runtime error: the program finished, but the output variable '{}' was never assigned.",
                    name
                )
            }


            EvalError::ParseError(message) => {
                write!(f, "Parse error: {}", message)
            }

            EvalError::TypeError(message) => {
                write!(f, "Type error: {}", message)
            }
            
            EvalError::TypeCheckError(message) => {
                write!(f, "Type checking error: {}", message)
            }

            EvalError::UnexpectedEndOfInput => {
                write!(f, "Parse error: unexpected end of input.")
            }
        }
    }
}


impl std::error::Error for EvalError {}