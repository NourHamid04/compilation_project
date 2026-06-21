// MiniImp program representation
#[derive(Debug, Clone)]
pub struct Program {
    pub input_var: String,
    pub output_var: String,
    pub body: Cmd,
}
// MiniImp commands
#[derive(Debug, Clone)]
pub enum Cmd {
    Skip,
    Assign(String, Expr),
    Seq(Box<Cmd>, Box<Cmd>),
    If(BoolExpr, Box<Cmd>, Box<Cmd>),
    While(BoolExpr, Box<Cmd>),
}
// Arithmetic and boolean expressions
#[derive(Debug, Clone)]
pub enum Expr {
    Var(String),
    Int(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BoolExpr {
    True,
    False,
    And(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
    Less(Box<Expr>, Box<Expr>),
}