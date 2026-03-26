use crate::common::error::EvalError;
use crate::minifun::ast::Term;
use crate::minifun::runtime::{Env, Value};

/// Evaluates a MiniFun term in the current environment.
///
/// It returns:
/// - Ok(value) if evaluation succeeds
/// - Err(EvalError) if something goes wrong, such as:
///   - reading an undefined variable
///   - applying arithmetic to non-integer values
///   - applying a non-function value


pub fn eval_term(env: &Env, term: &Term) -> Result<Value, EvalError> {
    match term {
        // Integer literals evaluate to integer values.
        Term::Int(value) => Ok(Value::Int(*value)),

        // Boolean literals evaluate to boolean values.
        Term::True => Ok(Value::Bool(true)),
        Term::False => Ok(Value::Bool(false)),

        // Variable lookup:
        // we search for the variable in the current environment.
        Term::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

        // Function values are evaluated as closures.
        // A closure stores:
        // 1. the parameter name
        // 2. the function body
        // 3. the current environment

        Term::Fun(parameter_name, body) => {
            Ok(Value::Closure(
                parameter_name.clone(),
                body.clone(),
                env.clone(),
            ))
        }

        // let x = t1 in t2
        //
        // 1. evaluate t1
        // 2. extend the environment with x
        // 3. evaluate t2 in the updated environment
        Term::Let(name, value_term, body_term) => {
            let value = eval_term(env, value_term)?;
            let mut updated_env = env.clone();
            updated_env.insert(name.clone(), value);
            eval_term(&updated_env, body_term)
        }

        // letfun f x = t1 in t2
        //
        // This creates a recursive closure and binds it to f.
        // Then t2 is evaluated in the extended environment.
        Term::LetFun(function_name, parameter_name, function_body, in_term) => {
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

        // if t1 then t2 else t3
        //
        // First evaluate the condition and it must produce a boolean value ,then choose the correct branch.
       
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

        // Function application: t1 t2
        //
        // 1. evaluate the function term
        // 2. evaluate the argument term
        // 3. apply the function if it is a closure
        Term::App(function_term, argument_term) => {
            let function_value = eval_term(env, function_term)?;
            let argument_value = eval_term(env, argument_term)?;

            match function_value {
                // bind the argument to the parameter in the saved closure environment,
                // then evaluate the body there.
                Value::Closure(parameter_name, body, closure_env) => {
                    let mut updated_env = closure_env.clone();
                    updated_env.insert(parameter_name, argument_value);
                    eval_term(&updated_env, &body)
                }

                // Recursive closure:
                // the function name must also be available in the environment,
                // so the body can call itself recursively.
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

                // Any other value cannot be applied like a function.
                _ => Err(EvalError::TypeError(
                    "attempted to apply a non-function value".to_string(),
                )),
            }
        }

        // For arithmetic operators, we first evaluate both sides.
        // Then we check that both values are integers.
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

        // Logical AND:
        // both sub-terms must evaluate to boolean values.
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
        // both sub-terms must evaluate to integer values.
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
        // the sub-term must evaluate to a boolean value.
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
///
/// Evaluation starts with an empty environment.
pub fn eval_program(term: &Term) -> Result<Value, EvalError> {
    let initial_env = Env::new();
    eval_term(&initial_env, term)
}