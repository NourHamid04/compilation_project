#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::miniimp::ast::{BoolExpr, Expr};
use crate::miniimp::cfg::{Block, Cfg, Edge, NodeId};

#[derive(Debug, Clone)]
pub struct VariableAnnotation {
    pub in_set: HashSet<String>,
    pub out_set: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct VariableAnnotatedNode {
    pub id: NodeId,
    pub block: Block,
    pub edge: Edge,
    pub annotation: VariableAnnotation,
}

#[derive(Debug, Clone)]
pub struct VariableAnnotatedCfg {
    pub nodes: Vec<VariableAnnotatedNode>,
    pub start: NodeId,
    pub end: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefinitionPlace {
    Input,
    Node(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub variable: String,
    pub place: DefinitionPlace,
}

#[derive(Debug, Clone)]
pub struct ReachingAnnotation {
    pub in_set: HashSet<Definition>,
    pub out_set: HashSet<Definition>,
}

#[derive(Debug, Clone)]
pub struct ReachingAnnotatedNode {
    pub id: NodeId,
    pub block: Block,
    pub edge: Edge,
    pub annotation: ReachingAnnotation,
}

#[derive(Debug, Clone)]
pub struct ReachingAnnotatedCfg {
    pub nodes: Vec<ReachingAnnotatedNode>,
    pub start: NodeId,
    pub end: NodeId,
}

// Gets the next nodes of one CFG node.
fn successors(edge: &Edge) -> Vec<NodeId> {
    match edge {
        Edge::End => vec![],
        Edge::Next(node) => vec![*node],
        Edge::If {
            true_branch,
            false_branch,
        } => vec![*true_branch, *false_branch],
    }
}

// Gets the previous nodes for each CFG node.
fn predecessors(cfg: &Cfg) -> HashMap<NodeId, Vec<NodeId>> {
    let mut result = HashMap::new();

    for node in &cfg.nodes {
        for next_node in successors(&node.edge) {
            result.entry(next_node).or_insert(Vec::new()).push(node.id);
        }
    }

    result
}

// Gets all variables used inside an arithmetic expression.
fn vars_in_expr(expr: &Expr) -> HashSet<String> {
    let mut vars = HashSet::new();

    match expr {
        Expr::Var(name) => {
            vars.insert(name.clone());
        }

        Expr::Int(_) => {}

        Expr::Add(left, right) => {
            vars.extend(vars_in_expr(left));
            vars.extend(vars_in_expr(right));
        }

        Expr::Sub(left, right) => {
            vars.extend(vars_in_expr(left));
            vars.extend(vars_in_expr(right));
        }

        Expr::Mul(left, right) => {
            vars.extend(vars_in_expr(left));
            vars.extend(vars_in_expr(right));
        }
    }

    vars
}

// Gets all variables used inside a boolean expression.
fn vars_in_bool(expr: &BoolExpr) -> HashSet<String> {
    let mut vars = HashSet::new();

    match expr {
        BoolExpr::True => {}

        BoolExpr::False => {}

        BoolExpr::Less(left, right) => {
            vars.extend(vars_in_expr(left));
            vars.extend(vars_in_expr(right));
        }

        BoolExpr::And(left, right) => {
            vars.extend(vars_in_bool(left));
            vars.extend(vars_in_bool(right));
        }

        BoolExpr::Not(inner) => {
            vars.extend(vars_in_bool(inner));
        }
    }

    vars
}

// Gets the variable assigned by a block, if there is one.
fn assigned_variable(block: &Block) -> Option<String> {
    match block {
        Block::Assignment(name, _) => Some(name.clone()),
        _ => None,
    }
}

// Gets the variables used by a block.
fn used_variables(block: &Block) -> HashSet<String> {
    match block {
        Block::Skip => HashSet::new(),

        Block::Assignment(_, expr) => vars_in_expr(expr),

        Block::Condition(condition) => vars_in_bool(condition),
    }
}

// Computes defined variables for the CFG.
pub fn defined_variables(cfg: &Cfg) -> VariableAnnotatedCfg {
    let preds = predecessors(cfg);

    let mut in_sets: HashMap<NodeId, HashSet<String>> = HashMap::new();
    let mut out_sets: HashMap<NodeId, HashSet<String>> = HashMap::new();

    for node in &cfg.nodes {
        in_sets.insert(node.id, HashSet::new());
        out_sets.insert(node.id, HashSet::new());
    }

    loop {
        let old_in_sets = in_sets.clone();
        let old_out_sets = out_sets.clone();

        for node in &cfg.nodes {
            let mut input_set = HashSet::new();

            if node.id == cfg.start {
                input_set.insert(cfg.input_var.clone());
            } else if let Some(previous_nodes) = preds.get(&node.id) {
                if !previous_nodes.is_empty() {
                    input_set = out_sets[&previous_nodes[0]].clone();

                    for previous in previous_nodes.iter().skip(1) {
                        input_set = input_set
                            .intersection(&out_sets[previous])
                            .cloned()
                            .collect();
                    }
                }
            }

            let mut output_set = input_set.clone();

            if let Some(variable) = assigned_variable(&node.block) {
                output_set.insert(variable);
            }

            in_sets.insert(node.id, input_set);
            out_sets.insert(node.id, output_set);
        }

        if old_in_sets == in_sets && old_out_sets == out_sets {
            break;
        }
    }

    let mut annotated_nodes = Vec::new();

    for node in &cfg.nodes {
        annotated_nodes.push(VariableAnnotatedNode {
            id: node.id,
            block: node.block.clone(),
            edge: node.edge.clone(),
            annotation: VariableAnnotation {
                in_set: in_sets[&node.id].clone(),
                out_set: out_sets[&node.id].clone(),
            },
        });
    }

    VariableAnnotatedCfg {
        nodes: annotated_nodes,
        start: cfg.start,
        end: cfg.end,
    }
}

// Computes live variables for the CFG.
pub fn live_variables(cfg: &Cfg) -> VariableAnnotatedCfg {
    let mut in_sets: HashMap<NodeId, HashSet<String>> = HashMap::new();
    let mut out_sets: HashMap<NodeId, HashSet<String>> = HashMap::new();

    for node in &cfg.nodes {
        in_sets.insert(node.id, HashSet::new());
        out_sets.insert(node.id, HashSet::new());
    }

    loop {
        let old_in_sets = in_sets.clone();
        let old_out_sets = out_sets.clone();

        for node in cfg.nodes.iter().rev() {
            let mut output_set = HashSet::new();

            for next_node in successors(&node.edge) {
                output_set.extend(in_sets[&next_node].clone());
            }

            if node.id == cfg.end {
                output_set.insert(cfg.output_var.clone());
            }

            let mut input_set = output_set.clone();

            if let Some(variable) = assigned_variable(&node.block) {
                input_set.remove(&variable);
            }

            input_set.extend(used_variables(&node.block));

            in_sets.insert(node.id, input_set);
            out_sets.insert(node.id, output_set);
        }

        if old_in_sets == in_sets && old_out_sets == out_sets {
            break;
        }
    }

    let mut annotated_nodes = Vec::new();

    for node in &cfg.nodes {
        annotated_nodes.push(VariableAnnotatedNode {
            id: node.id,
            block: node.block.clone(),
            edge: node.edge.clone(),
            annotation: VariableAnnotation {
                in_set: in_sets[&node.id].clone(),
                out_set: out_sets[&node.id].clone(),
            },
        });
    }

    VariableAnnotatedCfg {
        nodes: annotated_nodes,
        start: cfg.start,
        end: cfg.end,
    }
}

// Computes reaching definitions for the CFG.
pub fn reaching_definitions(cfg: &Cfg) -> ReachingAnnotatedCfg {
    let preds = predecessors(cfg);

    let mut in_sets: HashMap<NodeId, HashSet<Definition>> = HashMap::new();
    let mut out_sets: HashMap<NodeId, HashSet<Definition>> = HashMap::new();

    let input_definition = Definition {
        variable: cfg.input_var.clone(),
        place: DefinitionPlace::Input,
    };

    for node in &cfg.nodes {
        in_sets.insert(node.id, HashSet::new());
        out_sets.insert(node.id, HashSet::new());
    }

    loop {
        let old_in_sets = in_sets.clone();
        let old_out_sets = out_sets.clone();

        for node in &cfg.nodes {
            let mut input_set = HashSet::new();

            if node.id == cfg.start {
                input_set.insert(input_definition.clone());
            }

            if let Some(previous_nodes) = preds.get(&node.id) {
                for previous in previous_nodes {
                    input_set.extend(out_sets[previous].clone());
                }
            }

            let mut output_set = input_set.clone();

            if let Some(variable) = assigned_variable(&node.block) {
                output_set.retain(|definition| definition.variable != variable);

                output_set.insert(Definition {
                    variable,
                    place: DefinitionPlace::Node(node.id),
                });
            }

            in_sets.insert(node.id, input_set);
            out_sets.insert(node.id, output_set);
        }

        if old_in_sets == in_sets && old_out_sets == out_sets {
            break;
        }
    }

    let mut annotated_nodes = Vec::new();

    for node in &cfg.nodes {
        annotated_nodes.push(ReachingAnnotatedNode {
            id: node.id,
            block: node.block.clone(),
            edge: node.edge.clone(),
            annotation: ReachingAnnotation {
                in_set: in_sets[&node.id].clone(),
                out_set: out_sets[&node.id].clone(),
            },
        });
    }

    ReachingAnnotatedCfg {
        nodes: annotated_nodes,
        start: cfg.start,
        end: cfg.end,
    }
}