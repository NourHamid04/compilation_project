use std::fmt;

/// This enum contains all evaluation/runtime errors
/// that may happen while interpreting a MiniImp or MiniFun program.
///
/// Why do we need this?
/// Because not every program is valid at runtime.
/// For example:
/// - using a variable that was never assigned
/// - finishing execution without producing the expected output variable
///
/// Instead of crashing silently, we return a clear error message.
#[derive(Debug, Clone)]
pub enum EvalError {
    /// Raised when the interpreter tries to read a variable
    /// that does not exist in the current memory/environment.
    ///
    /// Example:
    ///     x := y + 1
    ///
    /// If `y` was never assigned before, then `y` is undefined.
    UndefinedVariable(String),

    /// Raised when the whole program finishes, but the declared
    /// output variable was never assigned any value.
    ///
    /// Example:
    /// If the program says the final result should be stored in `out`,
    /// but `out` does not exist in memory at the end, we report this error.
    OutputVariableUndefined(String),
    

    ParseError(String),
    UnexpectedEndOfInput,
    TypeError(String),

}

/// This implementation tells Rust how to convert our error
/// into a human-readable message.
///
/// It is useful when we print the error with:
///     println!("{}", error)
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


            EvalError::UnexpectedEndOfInput => {
                write!(f, "Parse error: unexpected end of input.")
            }
        }
    }
}

/// This allows EvalError to behave like a standard Rust error type.
/// It is not strictly necessary for very small projects,
/// but it is good Rust practice and keeps the code extensible.
impl std::error::Error for EvalError {}