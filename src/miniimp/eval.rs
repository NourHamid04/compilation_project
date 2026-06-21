use crate::common::error::EvalError;
use crate::miniimp::ast::{BoolExpr, Cmd, Expr, Program};
use crate::miniimp::runtime::Memory;

// Arithmetic expression evaluation

pub fn eval_expr(memory: &Memory, expr: &Expr) -> Result<i64, EvalError> {
    match expr {
        Expr::Int(value) => Ok(*value),

        // Variable:
   
        Expr::Var(name) => memory
            .get(name)
            .copied()
            .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

    // Arithmetic operators
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

// Boolean expression evaluation
pub fn eval_bool(memory: &Memory, bool_expr: &BoolExpr) -> Result<bool, EvalError> {
    match bool_expr {
        BoolExpr::True => Ok(true),
        BoolExpr::False => Ok(false),

// Boolean operators 
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

// Command evaluation

pub fn eval_cmd(memory: &Memory, cmd: &Cmd) -> Result<Memory, EvalError> {
    // Execute the command according to MiniImp semantics
    match cmd {
        Cmd::Skip => Ok(memory.clone()),

      
        Cmd::Assign(name, expr) => {
            let value = eval_expr(memory, expr)?;
            let mut updated_memory = memory.clone();
            updated_memory.insert(name.clone(), value);
            Ok(updated_memory)
        }

     
        Cmd::Seq(first, second) => {
            let intermediate_memory = eval_cmd(memory, first)?;
            eval_cmd(&intermediate_memory, second)
        }

  
        Cmd::If(condition, then_branch, else_branch) => {
            let condition_value = eval_bool(memory, condition)?;

            if condition_value {
                eval_cmd(memory, then_branch)
            } else {
                eval_cmd(memory, else_branch)
            }
        }


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

// Execute a complete MiniImp program

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