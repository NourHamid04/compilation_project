use crate::common::error::EvalError;
use crate::miniimp::ast::{BoolExpr, Cmd, Expr, Program};
use crate::miniimp::runtime::Memory;

/// Evaluates an arithmetic expression in the current memory.
///
/// It returns:
/// - Ok(number) if evaluation succeeds
/// - Err(EvalError) if something goes wrong, such as reading an undefined variable
pub fn eval_expr(memory: &Memory, expr: &Expr) -> Result<i64, EvalError> {
    match expr {
        // Integer literals evaluate to themselves.
        Expr::Int(value) => Ok(*value),

        // Variable lookup:
        // we search for the variable in memory.
        // If it is not found, we return a clear runtime error.
        Expr::Var(name) => memory
            .get(name)
            .copied()
            .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

        // For binary arithmetic operators, we first evaluate both sides,
        // then apply the corresponding operation.
        Expr::Add(left, right) => {
            let left_value = eval_expr(memory, left)?;
            let right_value = eval_expr(memory, right)?;
            Ok(left_value + right_value)
        }

        Expr::Sub(left, right) => {
            let left_value = eval_expr(memory, left)?;
            let right_value = eval_expr(memory, right)?;
            Ok(left_value - right_value)
        }

        Expr::Mul(left, right) => {
            let left_value = eval_expr(memory, left)?;
            let right_value = eval_expr(memory, right)?;
            Ok(left_value * right_value)
        }
    }
}

/// Evaluates a boolean expression in the current memory.
///
/// It returns:
/// - Ok(true/false) if evaluation succeeds
/// - Err(EvalError) if evaluation gets stuck because of an undefined variable
pub fn eval_bool(memory: &Memory, bool_expr: &BoolExpr) -> Result<bool, EvalError> {
    match bool_expr {
        BoolExpr::True => Ok(true),
        BoolExpr::False => Ok(false),

        // Logical AND:
        // both sub-expressions must be evaluated first.
        BoolExpr::And(left, right) => {
            let left_value = eval_bool(memory, left)?;
            let right_value = eval_bool(memory, right)?;
            Ok(left_value && right_value)
        }

        // Logical negation.
        BoolExpr::Not(expr) => {
            let value = eval_bool(memory, expr)?;
            Ok(!value)
        }

        // Less-than comparison between two arithmetic expressions.
        BoolExpr::Less(left, right) => {
            let left_value = eval_expr(memory, left)?;
            let right_value = eval_expr(memory, right)?;
            Ok(left_value < right_value)
        }
    }
}

/// Executes a command and returns the updated memory.
///
/// It returns:
/// - Ok(new_memory) if execution succeeds
/// - Err(EvalError) if execution fails during expression/condition evaluation
///
/// Note:
/// We return a new memory instead of modifying the existing one in place.
/// This keeps the code simple and close to the formal semantics.
pub fn eval_cmd(memory: &Memory, cmd: &Cmd) -> Result<Memory, EvalError> {
    match cmd {
        // skip does nothing, so memory stays unchanged.
        Cmd::Skip => Ok(memory.clone()),

        // Assignment:
        // 1. evaluate the expression
        // 2. copy the current memory
        // 3. store the new value for the target variable
        Cmd::Assign(name, expr) => {
            let value = eval_expr(memory, expr)?;
            let mut updated_memory = memory.clone();
            updated_memory.insert(name.clone(), value);
            Ok(updated_memory)
        }

        // Sequential composition:
        // execute the first command, then use its resulting memory
        // to execute the second command.
        Cmd::Seq(first, second) => {
            let intermediate_memory = eval_cmd(memory, first)?;
            eval_cmd(&intermediate_memory, second)
        }

        // If command:
        // evaluate the condition, then choose which branch to execute.
        Cmd::If(condition, then_branch, else_branch) => {
            let condition_value = eval_bool(memory, condition)?;

            if condition_value {
                eval_cmd(memory, then_branch)
            } else {
                eval_cmd(memory, else_branch)
            }
        }

        // While loop:
        // if the condition is true, execute the body once,
        // then continue evaluating the same loop with the new memory.
        // if the condition is false, stop and return the current memory.
        Cmd::While(condition, body) => {
            let condition_value = eval_bool(memory, condition)?;

            if condition_value {
                let updated_memory = eval_cmd(memory, body)?;
                eval_cmd(&updated_memory, cmd)
            } else {
                Ok(memory.clone())
            }
        }
    }
}

/// Runs a complete MiniImp program.
///
/// The program starts with an empty memory, except for the input variable,
/// which is initialized with the given input value.
///
/// At the end, the function tries to read the declared output variable
/// from the final memory and returns it.
pub fn eval_program(program: &Program, input_value: i64) -> Result<i64, EvalError> {
    let mut initial_memory = Memory::new();

    // The input variable is the only variable defined at the beginning.
    initial_memory.insert(program.input_var.clone(), input_value);

    let final_memory = eval_cmd(&initial_memory, &program.body)?;

    final_memory
        .get(&program.output_var)
        .copied()
        .ok_or_else(|| EvalError::OutputVariableUndefined(program.output_var.clone()))
}