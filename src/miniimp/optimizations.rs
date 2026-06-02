#![allow(dead_code)]

use crate::miniimp::ast::{BoolExpr, Expr};
use crate::miniimp::cfg::{Block, Cfg};
use crate::miniimp::dataflow::{
    defined_variables,
    live_variables,
    reaching_definitions,
    DefinitionPlace,
};

// Gets variables used in arithmetic expressions.
fn vars_in_expr(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::Var(name) => vec![name.clone()],
        Expr::Int(_) => vec![],

        Expr::Add(left, right)
        | Expr::Sub(left, right)
        | Expr::Mul(left, right) => {
            let mut vars = vars_in_expr(left);
            vars.extend(vars_in_expr(right));
            vars
        }
    }
}

// Gets variables used in boolean expressions.
fn vars_in_bool(expr: &BoolExpr) -> Vec<String> {
    match expr {
        BoolExpr::True | BoolExpr::False => vec![],

        BoolExpr::Less(left, right) => {
            let mut vars = vars_in_expr(left);
            vars.extend(vars_in_expr(right));
            vars
        }

        BoolExpr::And(left, right) => {
            let mut vars = vars_in_bool(left);
            vars.extend(vars_in_bool(right));
            vars
        }

        BoolExpr::Not(inner) => vars_in_bool(inner),
    }
}

// Gets variables used in one CFG block.
fn used_variables(block: &Block) -> Vec<String> {
    match block {
        Block::Skip => vec![],
        Block::Assignment(_, expr) => vars_in_expr(expr),
        Block::Condition(cond) => vars_in_bool(cond),
    }
}

// Gets the variable assigned in a block.
fn assigned_variable(block: &Block) -> Option<String> {
    match block {
        Block::Assignment(name, _) => Some(name.clone()),
        _ => None,
    }
}

// Checks possibly undefined variables using defined variables analysis.
pub fn check_undefined_variables(cfg: &Cfg) {
    let analysed_cfg = defined_variables(cfg);

    println!("Possibly undefined variables:");

    let mut found = false;

    for node in analysed_cfg.nodes {
        let used = used_variables(&node.block);

        for var in used {
            if !node.annotation.in_set.contains(&var) {
                println!(
                    "Variable '{}' may be undefined at node {}",
                    var, node.id
                );
                found = true;
            }
        }
    }

    if !found {
        println!("No possibly undefined variables found.");
    }
}

// Folds arithmetic constants.
fn fold_expr(expr: &Expr) -> (Expr, bool) {
    match expr {
        Expr::Int(_) => (expr.clone(), false),
        Expr::Var(_) => (expr.clone(), false),

        Expr::Add(left, right) => {
            let (left_new, left_changed) = fold_expr(left);
            let (right_new, right_changed) = fold_expr(right);

            match (&left_new, &right_new) {
                (Expr::Int(a), Expr::Int(b)) => (Expr::Int(a + b), true),
                (Expr::Int(0), _) => (right_new, true),
                (_, Expr::Int(0)) => (left_new, true),

                _ => (
                    Expr::Add(Box::new(left_new), Box::new(right_new)),
                    left_changed || right_changed,
                ),
            }
        }

        Expr::Sub(left, right) => {
            let (left_new, left_changed) = fold_expr(left);
            let (right_new, right_changed) = fold_expr(right);

            match (&left_new, &right_new) {
                (Expr::Int(a), Expr::Int(b)) => (Expr::Int(a - b), true),
                (_, Expr::Int(0)) => (left_new, true),

                _ => (
                    Expr::Sub(Box::new(left_new), Box::new(right_new)),
                    left_changed || right_changed,
                ),
            }
        }

        Expr::Mul(left, right) => {
            let (left_new, left_changed) = fold_expr(left);
            let (right_new, right_changed) = fold_expr(right);

            match (&left_new, &right_new) {
                (Expr::Int(a), Expr::Int(b)) => (Expr::Int(a * b), true),
                (Expr::Int(0), _) => (Expr::Int(0), true),
                (_, Expr::Int(0)) => (Expr::Int(0), true),
                (Expr::Int(1), _) => (right_new, true),
                (_, Expr::Int(1)) => (left_new, true),

                _ => (
                    Expr::Mul(Box::new(left_new), Box::new(right_new)),
                    left_changed || right_changed,
                ),
            }
        }
    }
}

// Folds boolean constants.
fn fold_bool(expr: &BoolExpr) -> (BoolExpr, bool) {
    match expr {
        BoolExpr::True => (BoolExpr::True, false),
        BoolExpr::False => (BoolExpr::False, false),

        BoolExpr::Less(left, right) => {
            let (left_new, left_changed) = fold_expr(left);
            let (right_new, right_changed) = fold_expr(right);

            match (&left_new, &right_new) {
                (Expr::Int(a), Expr::Int(b)) => {
                    if a < b {
                        (BoolExpr::True, true)
                    } else {
                        (BoolExpr::False, true)
                    }
                }

                _ => (
                    BoolExpr::Less(Box::new(left_new), Box::new(right_new)),
                    left_changed || right_changed,
                ),
            }
        }

        BoolExpr::And(left, right) => {
            let (left_new, left_changed) = fold_bool(left);
            let (right_new, right_changed) = fold_bool(right);

            match (&left_new, &right_new) {
                (BoolExpr::False, _) => (BoolExpr::False, true),
                (_, BoolExpr::False) => (BoolExpr::False, true),
                (BoolExpr::True, _) => (right_new, true),
                (_, BoolExpr::True) => (left_new, true),

                _ => (
                    BoolExpr::And(Box::new(left_new), Box::new(right_new)),
                    left_changed || right_changed,
                ),
            }
        }

        BoolExpr::Not(inner) => {
            let (inner_new, inner_changed) = fold_bool(inner);

            match inner_new {
                BoolExpr::True => (BoolExpr::False, true),
                BoolExpr::False => (BoolExpr::True, true),
                _ => (BoolExpr::Not(Box::new(inner_new)), inner_changed),
            }
        }
    }
}

// Applies constant folding to the CFG.
pub fn constant_folding(cfg: &Cfg) -> (Cfg, bool) {
    let mut new_cfg = cfg.clone();
    let mut changed = false;

    for node in &mut new_cfg.nodes {
        match &node.block {
            Block::Assignment(name, expr) => {
                let (new_expr, did_change) = fold_expr(expr);

                if did_change {
                    node.block = Block::Assignment(name.clone(), new_expr);
                    changed = true;
                }
            }

            Block::Condition(cond) => {
                let (new_cond, did_change) = fold_bool(cond);

                if did_change {
                    node.block = Block::Condition(new_cond);
                    changed = true;
                }
            }

            Block::Skip => {}
        }
    }

    (new_cfg, changed)
}

// Finds whether a variable has one constant reaching definition.
fn find_constant(
    cfg: &Cfg,
    node_id: usize,
    variable: &str,
) -> Option<i64> {
    let reaching_cfg = reaching_definitions(cfg);

    let current_node = reaching_cfg
        .nodes
        .iter()
        .find(|node| node.id == node_id)?;

    let mut found_node = None;

    for def in &current_node.annotation.in_set {
        if def.variable == variable {
            if found_node.is_some() {
                return None;
            }

            found_node = Some(def.clone());
        }
    }

    let def = found_node?;

    match def.place {
        DefinitionPlace::Input => None,

        DefinitionPlace::Node(def_node_id) => {
            let node = cfg.nodes.iter().find(|node| node.id == def_node_id)?;

            match &node.block {
                Block::Assignment(_, Expr::Int(value)) => Some(*value),
                _ => None,
            }
        }
    }
}

// Propagates constants in arithmetic expressions.
fn propagate_expr(cfg: &Cfg, node_id: usize, expr: &Expr) -> (Expr, bool) {
    match expr {
        Expr::Int(_) => (expr.clone(), false),

        Expr::Var(name) => {
            if let Some(value) = find_constant(cfg, node_id, name) {
                (Expr::Int(value), true)
            } else {
                (expr.clone(), false)
            }
        }

        Expr::Add(left, right) => {
            let (left_new, left_changed) = propagate_expr(cfg, node_id, left);
            let (right_new, right_changed) = propagate_expr(cfg, node_id, right);

            (
                Expr::Add(Box::new(left_new), Box::new(right_new)),
                left_changed || right_changed,
            )
        }

        Expr::Sub(left, right) => {
            let (left_new, left_changed) = propagate_expr(cfg, node_id, left);
            let (right_new, right_changed) = propagate_expr(cfg, node_id, right);

            (
                Expr::Sub(Box::new(left_new), Box::new(right_new)),
                left_changed || right_changed,
            )
        }

        Expr::Mul(left, right) => {
            let (left_new, left_changed) = propagate_expr(cfg, node_id, left);
            let (right_new, right_changed) = propagate_expr(cfg, node_id, right);

            (
                Expr::Mul(Box::new(left_new), Box::new(right_new)),
                left_changed || right_changed,
            )
        }
    }
}

// Propagates constants in boolean expressions.
fn propagate_bool(cfg: &Cfg, node_id: usize, expr: &BoolExpr) -> (BoolExpr, bool) {
    match expr {
        BoolExpr::True => (BoolExpr::True, false),
        BoolExpr::False => (BoolExpr::False, false),

        BoolExpr::Less(left, right) => {
            let (left_new, left_changed) = propagate_expr(cfg, node_id, left);
            let (right_new, right_changed) = propagate_expr(cfg, node_id, right);

            (
                BoolExpr::Less(Box::new(left_new), Box::new(right_new)),
                left_changed || right_changed,
            )
        }

        BoolExpr::And(left, right) => {
            let (left_new, left_changed) = propagate_bool(cfg, node_id, left);
            let (right_new, right_changed) = propagate_bool(cfg, node_id, right);

            (
                BoolExpr::And(Box::new(left_new), Box::new(right_new)),
                left_changed || right_changed,
            )
        }

        BoolExpr::Not(inner) => {
            let (inner_new, inner_changed) = propagate_bool(cfg, node_id, inner);

            (
                BoolExpr::Not(Box::new(inner_new)),
                inner_changed,
            )
        }
    }
}

// Applies constant propagation to the CFG.
pub fn constant_propagation(cfg: &Cfg) -> (Cfg, bool) {
    let mut new_cfg = cfg.clone();
    let mut changed = false;

    for node in &mut new_cfg.nodes {
        match &node.block {
            Block::Assignment(name, expr) => {
                let (new_expr, did_change) = propagate_expr(cfg, node.id, expr);

                if did_change {
                    node.block = Block::Assignment(name.clone(), new_expr);
                    changed = true;
                }
            }

            Block::Condition(cond) => {
                let (new_cond, did_change) = propagate_bool(cfg, node.id, cond);

                if did_change {
                    node.block = Block::Condition(new_cond);
                    changed = true;
                }
            }

            Block::Skip => {}
        }
    }

    (new_cfg, changed)
}

// Removes assignments whose variable is not live after the node.
pub fn dead_store_elimination(cfg: &Cfg) -> (Cfg, bool) {
    let live_cfg = live_variables(cfg);

    let mut new_cfg = cfg.clone();
    let mut changed = false;

    for node in live_cfg.nodes {
        if let Some(var) = assigned_variable(&node.block) {
            if !node.annotation.out_set.contains(&var) {
                new_cfg.nodes[node.id].block = Block::Skip;
                changed = true;
            }
        }
    }

    (new_cfg, changed)
}

// Runs the optimization passes in a simple order.
pub fn optimize_cfg(cfg: &Cfg) -> Cfg {
    let mut current = cfg.clone();

    for _ in 0..10 {
        let mut changed = false;

        let (next, did_change) = constant_propagation(&current);
        current = next;
        changed = changed || did_change;

        let (next, did_change) = constant_folding(&current);
        current = next;
        changed = changed || did_change;

        if !changed {
            break;
        }
    }

    let (current, _) = dead_store_elimination(&current);

    current
}