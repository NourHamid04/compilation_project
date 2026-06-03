use crate::miniimp::ast::{BoolExpr, Cmd, Expr, Program};
use crate::miniimp::cfg::{build_cfg, Block, Edge};

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io;

pub fn write_llvm_ir(program: &Program, file_name: &str) -> io::Result<()> {
    let llvm_code = generate_llvm_ir(program);
    fs::write(file_name, llvm_code)
}

pub fn generate_llvm_ir(program: &Program) -> String {
    let cfg = build_cfg(program);

    let mut code = String::new();
    let mut counter = 0;

    let variables = collect_variables(program);
    let mut memory_names = HashMap::new();

    for variable in &variables {
        memory_names.insert(variable.clone(), format!("%{}_ptr", variable));
    }

    code.push_str("define i64 @func(i64 %in) {\n");
    code.push_str("entry:\n");

    // Allocate one stack slot for each source variable.
    for variable in &variables {
        let pointer = memory_names.get(variable).unwrap();
        code.push_str(&format!("  {} = alloca i64\n", pointer));
    }

    // Store the function input in the input variable.
    for variable in &variables {
        let pointer = memory_names.get(variable).unwrap();

        if variable == &cfg.input_var {
            code.push_str(&format!("  store i64 %in, ptr {}\n", pointer));
        } else {
            code.push_str(&format!("  store i64 0, ptr {}\n", pointer));
        }
    }

    code.push_str(&format!("  br label %block{}\n", cfg.start));

    for node in &cfg.nodes {
        code.push_str(&format!("block{}:\n", node.id));

        match &node.block {
            Block::Skip => {}

            Block::Assignment(variable, expression) => {
                let value = generate_expr(
                    expression,
                    &memory_names,
                    &mut code,
                    &mut counter,
                );

                let pointer = memory_names.get(variable).unwrap();
                code.push_str(&format!("  store i64 {}, ptr {}\n", value, pointer));
            }

            Block::Condition(condition) => {
                let condition_value = generate_bool_expr(
                    condition,
                    &memory_names,
                    &mut code,
                    &mut counter,
                );

                if let Edge::If {
                    true_branch,
                    false_branch,
                } = &node.edge
                {
                    code.push_str(&format!(
                        "  br i1 {}, label %block{}, label %block{}\n",
                        condition_value, true_branch, false_branch
                    ));

                    continue;
                }
            }
        }

        match &node.edge {
            Edge::End => {
                let pointer = memory_names.get(&cfg.output_var).unwrap();
                let output_value = fresh_name(&mut counter);

                code.push_str(&format!("  {} = load i64, ptr {}\n", output_value, pointer));
                code.push_str(&format!("  ret i64 {}\n", output_value));
            }

            Edge::Next(next_node) => {
                code.push_str(&format!("  br label %block{}\n", next_node));
            }

            Edge::If {
                true_branch,
                false_branch,
            } => {
                code.push_str(&format!(
                    "  br label %block{} ; unused fallback to block{}\n",
                    true_branch, false_branch
                ));
            }
        }
    }

    code.push_str("}\n");
    code
}

fn generate_expr(
    expression: &Expr,
    memory_names: &HashMap<String, String>,
    code: &mut String,
    counter: &mut usize,
) -> String {
    match expression {
        Expr::Int(number) => number.to_string(),

        Expr::Var(variable) => {
            let pointer = memory_names.get(variable).unwrap();
            let temp = fresh_name(counter);

            code.push_str(&format!("  {} = load i64, ptr {}\n", temp, pointer));

            temp
        }

        Expr::Add(left, right) => {
            let left_value = generate_expr(left, memory_names, code, counter);
            let right_value = generate_expr(right, memory_names, code, counter);
            let temp = fresh_name(counter);

            code.push_str(&format!(
                "  {} = add i64 {}, {}\n",
                temp, left_value, right_value
            ));

            temp
        }

        Expr::Sub(left, right) => {
            let left_value = generate_expr(left, memory_names, code, counter);
            let right_value = generate_expr(right, memory_names, code, counter);
            let temp = fresh_name(counter);

            code.push_str(&format!(
                "  {} = sub i64 {}, {}\n",
                temp, left_value, right_value
            ));

            temp
        }

        Expr::Mul(left, right) => {
            let left_value = generate_expr(left, memory_names, code, counter);
            let right_value = generate_expr(right, memory_names, code, counter);
            let temp = fresh_name(counter);

            code.push_str(&format!(
                "  {} = mul i64 {}, {}\n",
                temp, left_value, right_value
            ));

            temp
        }
    }
}

fn generate_bool_expr(
    condition: &BoolExpr,
    memory_names: &HashMap<String, String>,
    code: &mut String,
    counter: &mut usize,
) -> String {
    match condition {
        BoolExpr::True => "true".to_string(),

        BoolExpr::False => "false".to_string(),

        BoolExpr::Less(left, right) => {
            let left_value = generate_expr(left, memory_names, code, counter);
            let right_value = generate_expr(right, memory_names, code, counter);
            let temp = fresh_name(counter);

            code.push_str(&format!(
                "  {} = icmp slt i64 {}, {}\n",
                temp, left_value, right_value
            ));

            temp
        }

        BoolExpr::Not(inner_condition) => {
            let value = generate_bool_expr(inner_condition, memory_names, code, counter);
            let temp = fresh_name(counter);

            code.push_str(&format!("  {} = xor i1 {}, true\n", temp, value));

            temp
        }

        BoolExpr::And(left, right) => {
            let left_value = generate_bool_expr(left, memory_names, code, counter);
            let right_value = generate_bool_expr(right, memory_names, code, counter);
            let temp = fresh_name(counter);

            code.push_str(&format!(
                "  {} = and i1 {}, {}\n",
                temp, left_value, right_value
            ));

            temp
        }
    }
}

fn fresh_name(counter: &mut usize) -> String {
    let name = format!("%tmp{}", counter);
    *counter += 1;
    name
}

fn collect_variables(program: &Program) -> BTreeSet<String> {
    let mut variables = BTreeSet::new();

    variables.insert(program.input_var.clone());
    variables.insert(program.output_var.clone());

    collect_variables_from_cmd(&program.body, &mut variables);

    variables
}

fn collect_variables_from_cmd(command: &Cmd, variables: &mut BTreeSet<String>) {
    match command {
        Cmd::Skip => {}

        Cmd::Assign(variable, expression) => {
            variables.insert(variable.clone());
            collect_variables_from_expr(expression, variables);
        }

        Cmd::Seq(first, second) => {
            collect_variables_from_cmd(first, variables);
            collect_variables_from_cmd(second, variables);
        }

        Cmd::If(condition, then_command, else_command) => {
            collect_variables_from_bool(condition, variables);
            collect_variables_from_cmd(then_command, variables);
            collect_variables_from_cmd(else_command, variables);
        }

        Cmd::While(condition, body) => {
            collect_variables_from_bool(condition, variables);
            collect_variables_from_cmd(body, variables);
        }
    }
}

fn collect_variables_from_expr(expression: &Expr, variables: &mut BTreeSet<String>) {
    match expression {
        Expr::Var(variable) => {
            variables.insert(variable.clone());
        }

        Expr::Int(_) => {}

        Expr::Add(left, right) | Expr::Sub(left, right) | Expr::Mul(left, right) => {
            collect_variables_from_expr(left, variables);
            collect_variables_from_expr(right, variables);
        }
    }
}

fn collect_variables_from_bool(condition: &BoolExpr, variables: &mut BTreeSet<String>) {
    match condition {
        BoolExpr::True => {}

        BoolExpr::False => {}

        BoolExpr::And(left, right) => {
            collect_variables_from_bool(left, variables);
            collect_variables_from_bool(right, variables);
        }

        BoolExpr::Not(inner_condition) => {
            collect_variables_from_bool(inner_condition, variables);
        }

        BoolExpr::Less(left, right) => {
            collect_variables_from_expr(left, variables);
            collect_variables_from_expr(right, variables);
        }
    }
}