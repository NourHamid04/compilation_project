#![allow(dead_code)]
use crate::miniimp::ast::{BoolExpr, Cmd, Expr, Program};

pub type NodeId = usize;

#[derive(Debug, Clone)]
pub enum Block {
    Skip,
    Assignment(String, Expr),
    Condition(BoolExpr),
}

#[derive(Debug, Clone)]
pub enum Edge {
    End,
    Next(NodeId),
    If {
        true_branch: NodeId,
        false_branch: NodeId,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub block: Block,
    pub edge: Edge,
}

#[derive(Debug, Clone)]
pub struct Cfg {
    pub nodes: Vec<Node>,
    pub start: NodeId,
    pub end: NodeId,
}

fn new_node(nodes: &mut Vec<Node>, block: Block) -> NodeId {
    let id = nodes.len();

    nodes.push(Node {
        id,
        block,
        edge: Edge::End,
    });

    id
}

fn build_command(nodes: &mut Vec<Node>, command: &Cmd) -> (NodeId, NodeId) {
    match command {
        Cmd::Skip => {
            let node = new_node(nodes, Block::Skip);
            (node, node)
        }

        Cmd::Assign(variable, expression) => {
            let node = new_node(
                nodes,
                Block::Assignment(variable.clone(), expression.clone()),
            );
            (node, node)
        }

        Cmd::Seq(first, second) => {
            let (first_start, first_end) = build_command(nodes, first);
            let (second_start, second_end) = build_command(nodes, second);

            nodes[first_end].edge = Edge::Next(second_start);

            (first_start, second_end)
        }

        Cmd::If(condition, then_command, else_command) => {
            let condition_node = new_node(nodes, Block::Condition(condition.clone()));

            let (then_start, then_end) = build_command(nodes, then_command);
            let (else_start, else_end) = build_command(nodes, else_command);

            let after_if = new_node(nodes, Block::Skip);

            nodes[condition_node].edge = Edge::If {
                true_branch: then_start,
                false_branch: else_start,
            };

            nodes[then_end].edge = Edge::Next(after_if);
            nodes[else_end].edge = Edge::Next(after_if);

            (condition_node, after_if)
        }

        Cmd::While(condition, body) => {
            let condition_node = new_node(nodes, Block::Condition(condition.clone()));

            let (body_start, body_end) = build_command(nodes, body);

            let after_loop = new_node(nodes, Block::Skip);

            nodes[condition_node].edge = Edge::If {
                true_branch: body_start,
                false_branch: after_loop,
            };

            nodes[body_end].edge = Edge::Next(condition_node);

            (condition_node, after_loop)
        }
    }
}

pub fn build_cfg(program: &Program) -> Cfg {
    let mut nodes = Vec::new();

    let (start, end) = build_command(&mut nodes, &program.body);

    Cfg { nodes, start, end }
}