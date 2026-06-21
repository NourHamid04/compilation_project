use crate::common::error::EvalError;
use crate::minifun::ast::Term;
use crate::minifun::runtime::{Env, Value};

// Evaluate a MiniFun term in the given environment
pub fn eval_term(env: &Env, term: &Term) -> Result<Value, EvalError> {
    match term {
// Basic values and variables
        Term::Int(value) => Ok(Value::Int(*value)),
        Term::True => Ok(Value::Bool(true)),
        Term::False => Ok(Value::Bool(false)),
        Term::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

 

        Term::Fun(parameter_name, _parameter_type, body) => {
                Ok(Value::Closure(
                parameter_name.clone(),
                body.clone(),
                env.clone(),
            ))
        }

 // Functions, let-bindings, and recursive functions
        Term::Let(name, value_term, body_term) => {
            let value = eval_term(env, value_term)?;
            let mut updated_env = env.clone();
            updated_env.insert(name.clone(), value);
            eval_term(&updated_env, body_term)
        }

        Term::LetFun(function_name, parameter_name, _function_type, function_body, in_term) => {
                let mut updated_env = env.clone();

            let recursive_closure = Value::RecClosure(
                function_name.clone(),
                parameter_name.clone(),
                function_body.clone(),
                env.clone(),
            );

            updated_env.insert(function_name.clone(), recursive_closure);
            eval_term(&updated_env, in_term)
        }

  // Conditional expression
       
        Term::If(condition, then_term, else_term) => {
            let condition_value = eval_term(env, condition)?;

            match condition_value {
                Value::Bool(true) => eval_term(env, then_term),
                Value::Bool(false) => eval_term(env, else_term),
                _ => Err(EvalError::TypeError(
                    "if condition must evaluate to a boolean".to_string(),
                )),
            }
        }

 // Function application
        Term::App(function_term, argument_term) => {
            let function_value = eval_term(env, function_term)?;
            let argument_value = eval_term(env, argument_term)?;

            match function_value {
                // bind the argument to the parameter in the saved closure environment
                Value::Closure(parameter_name, body, closure_env) => {
                    let mut updated_env = closure_env.clone();
                    updated_env.insert(parameter_name, argument_value);
                    eval_term(&updated_env, &body)
                }

                // Recursive closure:
             
                Value::RecClosure(function_name, parameter_name, body, closure_env) => {
                    let mut updated_env = closure_env.clone();

                    let recursive_closure = Value::RecClosure(
                        function_name.clone(),
                        parameter_name.clone(),
                        body.clone(),
                        closure_env.clone(),
                    );

                    updated_env.insert(function_name, recursive_closure);
                    updated_env.insert(parameter_name, argument_value);

                    eval_term(&updated_env, &body)
                }

                _ => Err(EvalError::TypeError(
                    "attempted to apply a non-function value".to_string(),
                )),
            }
        }

// Arithmetic operators
        Term::Add(left, right) => {
            let left_value = eval_term(env, left)?;
            let right_value = eval_term(env, right)?;

            match (left_value, right_value) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                _ => Err(EvalError::TypeError(
                    "operator '+' expects two integers".to_string(),
                )),
            }
        }

        Term::Sub(left, right) => {
            let left_value = eval_term(env, left)?;
            let right_value = eval_term(env, right)?;

            match (left_value, right_value) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                _ => Err(EvalError::TypeError(
                    "operator '-' expects two integers".to_string(),
                )),
            }
        }

        Term::Mul(left, right) => {
            let left_value = eval_term(env, left)?;
            let right_value = eval_term(env, right)?;

            match (left_value, right_value) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                _ => Err(EvalError::TypeError(
                    "operator '*' expects two integers".to_string(),
                )),
            }
        }
// Boolean operators and comparisons
        // Logical AND:
        Term::And(left, right) => {
            let left_value = eval_term(env, left)?;
            let right_value = eval_term(env, right)?;

            match (left_value, right_value) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                _ => Err(EvalError::TypeError(
                    "operator 'and' expects two booleans".to_string(),
                )),
            }
        }

        // Less-than comparison:
        Term::Less(left, right) => {
            let left_value = eval_term(env, left)?;
            let right_value = eval_term(env, right)?;

            match (left_value, right_value) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                _ => Err(EvalError::TypeError(
                    "operator '<' expects two integers".to_string(),
                )),
            }
        }

        // Logical negation:
        Term::Not(term) => {
            let value = eval_term(env, term)?;

            match value {
                Value::Bool(boolean_value) => Ok(Value::Bool(!boolean_value)),
                _ => Err(EvalError::TypeError(
                    "operator 'not' expects a boolean".to_string(),
                )),
            }
        }
    }
}

/// Runs a complete MiniFun term.
pub fn eval_program(term: &Term) -> Result<Value, EvalError> {
    let initial_env = Env::new();
    eval_term(&initial_env, term)
}