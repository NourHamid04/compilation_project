use crate::common::error::EvalError;
use crate::miniimp::ast::{BoolExpr, Cmd, Expr};
use crate::miniimp::lexer::Token;

// Parser state: token list and current position

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}
// Basic token navigation helpers

impl Parser {
    /// Creates a new parser from a token list.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Returns the current token without consuming it.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Moves to the next token and returns the current one.
    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Checks whether the current token matches the expected token.
    fn check(&self, expected: &Token) -> bool {
        match self.peek() {
            Some(token) => token == expected,
            None => false,
        }
    }

    /// Consumes a token only if it matches the expected one.
    fn expect(&mut self, expected: Token) -> Result<(), EvalError> {
        match self.advance() {
            Some(token) if token == expected => Ok(()),
            Some(token) => Err(EvalError::ParseError(format!(
                "expected token {:?}, but found {:?}",
                expected, token
            ))),
            None => Err(EvalError::UnexpectedEndOfInput),
        }
    }

// Command parsing

    pub fn parse_cmd(&mut self) -> Result<Cmd, EvalError> {
        let mut cmd = self.parse_simple_cmd()?;

        while self.check(&Token::Semicolon) {
            self.advance(); // consume ';'
            let next_cmd = self.parse_simple_cmd()?;
            cmd = Cmd::Seq(Box::new(cmd), Box::new(next_cmd));
        }

        Ok(cmd)
    }

    /// Parses a single command that is not split by ';'
    fn parse_simple_cmd(&mut self) -> Result<Cmd, EvalError> {
        match self.peek() {
            Some(Token::Skip) => {
                self.advance();
                Ok(Cmd::Skip)
            }

            Some(Token::If) => {
                self.advance(); // consume 'if'
                let condition = self.parse_bool_expr()?;
                self.expect(Token::Then)?;
                let then_branch = self.parse_cmd()?;
                self.expect(Token::Else)?;
                let else_branch = self.parse_cmd()?;

                Ok(Cmd::If(
                    condition,
                    Box::new(then_branch),
                    Box::new(else_branch),
                ))
            }

            Some(Token::While) => {
                self.advance(); // consume 'while'
                let condition = self.parse_bool_expr()?;
                self.expect(Token::Do)?;
                let body = self.parse_cmd()?;

                Ok(Cmd::While(condition, Box::new(body)))
            }

            Some(Token::LParen) => {
                self.advance(); // consume '('
                let cmd = self.parse_cmd()?;
                self.expect(Token::RParen)?;
                Ok(cmd)
            }

            Some(Token::Identifier(_)) => self.parse_assignment(),

            Some(token) => Err(EvalError::ParseError(format!(
                "unexpected token {:?} while parsing a command",
                token
            ))),

            None => Err(EvalError::UnexpectedEndOfInput),
        }
    }

    /// Parses an assignment:
    fn parse_assignment(&mut self) -> Result<Cmd, EvalError> {
        let variable_name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            Some(token) => {
                return Err(EvalError::ParseError(format!(
                    "expected an identifier in assignment, but found {:?}",
                    token
                )));
            }
            None => return Err(EvalError::UnexpectedEndOfInput),
        };

        self.expect(Token::Assign)?;
        let expr = self.parse_expr()?;

        Ok(Cmd::Assign(variable_name, expr))
    }

// Arithmetic expression parsing
    pub fn parse_expr(&mut self) -> Result<Expr, EvalError> {
        self.parse_add_sub()
    }

    /// Parses + and - with left associativity.
    fn parse_add_sub(&mut self) -> Result<Expr, EvalError> {
        let mut expr = self.parse_mul()?;

        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_mul()?;
                    expr = Expr::Add(Box::new(expr), Box::new(right));
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_mul()?;
                    expr = Expr::Sub(Box::new(expr), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parses * with higher precedence than + and -.
    fn parse_mul(&mut self) -> Result<Expr, EvalError> {
        let mut expr = self.parse_primary()?;

        while self.check(&Token::Star) {
            self.advance();
            let right = self.parse_primary()?;
            expr = Expr::Mul(Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    /// Parses the smallest arithmetic units:
    /// - numbers
    /// - variables
    /// - parenthesized expressions
    fn parse_primary(&mut self) -> Result<Expr, EvalError> {
        match self.advance() {
            Some(Token::Number(value)) => Ok(Expr::Int(value)),

            Some(Token::Identifier(name)) => Ok(Expr::Var(name)),

            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }

            Some(token) => Err(EvalError::ParseError(format!(
                "unexpected token {:?} while parsing an arithmetic expression",
                token
            ))),

            None => Err(EvalError::UnexpectedEndOfInput),
        }
    }

// Boolean expression parsing
    pub fn parse_bool_expr(&mut self) -> Result<BoolExpr, EvalError> {
        self.parse_bool_and()
    }

    /// Parses `and` with left associativity.
    fn parse_bool_and(&mut self) -> Result<BoolExpr, EvalError> {
        let mut expr = self.parse_bool_not()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_bool_not()?;
            expr = BoolExpr::And(Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    /// Parses `not`.
    fn parse_bool_not(&mut self) -> Result<BoolExpr, EvalError> {
        if self.check(&Token::Not) {
            self.advance();
            let inner = self.parse_bool_not()?;
            Ok(BoolExpr::Not(Box::new(inner)))
        } else {
            self.parse_bool_primary()
        }
    }


    fn parse_bool_primary(&mut self) -> Result<BoolExpr, EvalError> {
        match self.peek() {
            Some(Token::True) => {
                self.advance();
                Ok(BoolExpr::True)
            }

            Some(Token::False) => {
                self.advance();
                Ok(BoolExpr::False)
            }

            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_bool_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }

            _ => {
                let left = self.parse_expr()?;

                if self.check(&Token::Less) {
                    self.advance();
                    let right = self.parse_expr()?;
                    Ok(BoolExpr::Less(Box::new(left), Box::new(right)))
                } else {
                    Err(EvalError::ParseError(
                        "expected a boolean expression or a comparison using '<'".to_string(),
                    ))
                }
            }
        }
    }

// Parse the full input and reject extra tokens

    pub fn finish(mut self) -> Result<Cmd, EvalError> {
        let cmd = self.parse_cmd()?;

        if self.peek().is_some() {
            return Err(EvalError::ParseError(format!(
                "unexpected extra token {:?} after the end of the command",
                self.peek()
            )));
        }

        Ok(cmd)
    }





}

// Public parser entry point
    pub fn parse_tokens(tokens: Vec<Token>) -> Result<Cmd, EvalError> {
        let parser = Parser::new(tokens);
        parser.finish()
    }
