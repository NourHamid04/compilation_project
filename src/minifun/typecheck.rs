use crate::common::error::EvalError;
use crate::minifun::ast::Term;
use crate::minifun::types::{Type, TypeEnv};

/// Typechecks a MiniFun term in the given typing environment.
///
/// It returns:
/// - Ok(type) if the term is well-typed
/// - Err(EvalError) if the term is not well-typed
pub fn typecheck_term(env: &TypeEnv, term: &Term) -> Result<Type, EvalError> {
    match term {
        Term::Int(_) => Ok(Type::Int),

        Term::True => Ok(Type::Bool),
        Term::False => Ok(Type::Bool),

        Term::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| {
                EvalError::TypeCheckError(format!(
                    "variable '{}' is not defined in the typing environment",
                    name
                ))
            }),

        Term::Add(left, right) => {
            let left_type = typecheck_term(env, left)?;
            let right_type = typecheck_term(env, right)?;

            if left_type == Type::Int && right_type == Type::Int {
                Ok(Type::Int)
            } else {
                Err(EvalError::TypeCheckError(
                    "operator '+' expects two integers".to_string(),
                ))
            }
        }

        Term::Sub(left, right) => {
            let left_type = typecheck_term(env, left)?;
            let right_type = typecheck_term(env, right)?;

            if left_type == Type::Int && right_type == Type::Int {
                Ok(Type::Int)
            } else {
                Err(EvalError::TypeCheckError(
                    "operator '-' expects two integers".to_string(),
                ))
            }
        }

        Term::Mul(left, right) => {
            let left_type = typecheck_term(env, left)?;
            let right_type = typecheck_term(env, right)?;

            if left_type == Type::Int && right_type == Type::Int {
                Ok(Type::Int)
            } else {
                Err(EvalError::TypeCheckError(
                    "operator '*' expects two integers".to_string(),
                ))
            }
        }

        Term::And(left, right) => {
            let left_type = typecheck_term(env, left)?;
            let right_type = typecheck_term(env, right)?;

            if left_type == Type::Bool && right_type == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(EvalError::TypeCheckError(
                    "operator '&&' expects two booleans".to_string(),
                ))
            }
        }

        Term::Less(left, right) => {
            let left_type = typecheck_term(env, left)?;
            let right_type = typecheck_term(env, right)?;

            if left_type == Type::Int && right_type == Type::Int {
                Ok(Type::Bool)
            } else {
                Err(EvalError::TypeCheckError(
                    "operator '<' expects two integers".to_string(),
                ))
            }
        }

        Term::Not(inner) => {
            let inner_type = typecheck_term(env, inner)?;

            if inner_type == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(EvalError::TypeCheckError(
                    "operator '~' expects a boolean".to_string(),
                ))
            }
        }

        Term::If(condition, then_term, else_term) => {
            let condition_type = typecheck_term(env, condition)?;
            let then_type = typecheck_term(env, then_term)?;
            let else_type = typecheck_term(env, else_term)?;

            if condition_type != Type::Bool {
                return Err(EvalError::TypeCheckError(
                    "if condition must have type bool".to_string(),
                ));
            }

            if then_type != else_type {
                return Err(EvalError::TypeCheckError(format!(
                    "if branches must have the same type, but found {:?} and {:?}",
                    then_type, else_type
                )));
            }

            Ok(then_type)
        }

        // fun x : tau => body
        Term::Fun(parameter_name, parameter_type, body) => {
            let mut updated_env = env.clone();
            updated_env.insert(parameter_name.clone(), parameter_type.clone());

            let body_type = typecheck_term(&updated_env, body)?;

            Ok(Type::Fun(
                Box::new(parameter_type.clone()),
                Box::new(body_type),
            ))
        }

        // t1 t2
        Term::App(function_term, argument_term) => {
            let function_type = typecheck_term(env, function_term)?;
            let argument_type = typecheck_term(env, argument_term)?;

            match function_type {
                Type::Fun(parameter_type, result_type) => {
                    if *parameter_type == argument_type {
                        Ok(*result_type)
                    } else {
                        Err(EvalError::TypeCheckError(format!(
                            "function expects argument of type {:?}, but found {:?}",
                            parameter_type, argument_type
                        )))
                    }
                }
                _ => Err(EvalError::TypeCheckError(
                    "attempted to apply a non-function term".to_string(),
                )),
            }
        }

        // let x = t1 in t2
        Term::Let(name, value_term, body_term) => {
            let value_type = typecheck_term(env, value_term)?;
            let mut updated_env = env.clone();
            updated_env.insert(name.clone(), value_type);
            typecheck_term(&updated_env, body_term)
        }

        // letfun f x : tau = t1 in t2
        Term::LetFun(function_name, parameter_name, function_type, function_body, in_term) => {
            match function_type {
                Type::Fun(parameter_type, return_type) => {
                    let mut function_env = env.clone();
                    function_env.insert(function_name.clone(), function_type.clone());
                    function_env.insert(parameter_name.clone(), (*parameter_type.clone()).clone());

                    let body_type = typecheck_term(&function_env, function_body)?;

                    if body_type != *return_type.clone() {
                        return Err(EvalError::TypeCheckError(format!(
                            "letfun body has type {:?}, but the annotation expects {:?}",
                            body_type, return_type
                        )));
                    }

                    let mut in_env = env.clone();
                    in_env.insert(function_name.clone(), function_type.clone());

                    typecheck_term(&in_env, in_term)
                }

                _ => Err(EvalError::TypeCheckError(
                    "letfun annotation must be a function type".to_string(),
                )),
            }
        }
    }
}

/// Typechecks a complete MiniFun term starting from an empty typing environment.
pub fn typecheck_program(term: &Term) -> Result<Type, EvalError> {
    let initial_env = TypeEnv::new();
    typecheck_term(&initial_env, term)
}