use crate::common::error::EvalError;
use crate::minifun::ast::Term;
use crate::minifun::lexer::Token;
use crate::minifun::types::Type;
/// A simple recursive-descent parser for MiniFun.
///
/// The parser reads a list of tokens and builds the corresponding AST.
/// It keeps track of the current token using `position`.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

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

    /// Entry point for MiniFun terms.
    pub fn parse_term(&mut self) -> Result<Term, EvalError> {
        match self.peek() {
            Some(Token::If) => self.parse_if(),
            Some(Token::Let) => self.parse_let(),
            Some(Token::LetFun) => self.parse_letfun(),
            Some(Token::Fun) => self.parse_fun(),
            _ => self.parse_and(),
        }
    }

    /// Parses:
    /// if t1 then t2 else t3
    fn parse_if(&mut self) -> Result<Term, EvalError> {
        self.expect(Token::If)?;
        let condition = self.parse_term()?;
        self.expect(Token::Then)?;
        let then_term = self.parse_term()?;
        self.expect(Token::Else)?;
        let else_term = self.parse_term()?;

        Ok(Term::If(
            Box::new(condition),
            Box::new(then_term),
            Box::new(else_term),
        ))
    }

    /// Parses:
    /// let x = t1 in t2
    fn parse_let(&mut self) -> Result<Term, EvalError> {
        self.expect(Token::Let)?;

        let name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            Some(token) => {
                return Err(EvalError::ParseError(format!(
                    "expected an identifier after 'let', but found {:?}",
                    token
                )));
            }
            None => return Err(EvalError::UnexpectedEndOfInput),
        };

        self.expect(Token::Equal)?;
        let value_term = self.parse_term()?;
        self.expect(Token::In)?;
        let body_term = self.parse_term()?;

        Ok(Term::Let(
            name,
            Box::new(value_term),
            Box::new(body_term),
        ))
    }

    /// Parses:
    /// letfun f x = t1 in t2
    fn parse_letfun(&mut self) -> Result<Term, EvalError> {
        self.expect(Token::LetFun)?;

        let function_name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            Some(token) => {
                return Err(EvalError::ParseError(format!(
                    "expected a function name after 'letfun', but found {:?}",
                    token
                )));
            }
            None => return Err(EvalError::UnexpectedEndOfInput),
        };

        let parameter_name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            Some(token) => {
                return Err(EvalError::ParseError(format!(
                    "expected a parameter name in 'letfun', but found {:?}",
                    token
                )));
            }
            None => return Err(EvalError::UnexpectedEndOfInput),
        };

        self.expect(Token::Colon)?;
        let function_type = self.parse_type()?;

        self.expect(Token::Equal)?;
        let function_body = self.parse_term()?;
        self.expect(Token::In)?;
        let in_term = self.parse_term()?;

        Ok(Term::LetFun(
            function_name,
            parameter_name,
            function_type,
            Box::new(function_body),
            Box::new(in_term),
        ))
    }

    /// Parses:
    /// fun x => t
    fn parse_fun(&mut self) -> Result<Term, EvalError> {
        self.expect(Token::Fun)?;

        let parameter_name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            Some(token) => {
                return Err(EvalError::ParseError(format!(
                    "expected a parameter name after 'fun', but found {:?}",
                    token
                )));
            }
            None => return Err(EvalError::UnexpectedEndOfInput),
        };

        self.expect(Token::Colon)?;
        let parameter_type = self.parse_type()?;

        self.expect(Token::Arrow)?;
        let body = self.parse_term()?;

        Ok(Term::Fun(parameter_name, parameter_type, Box::new(body)))
    }

    /// Parses `and` with left associativity.
    fn parse_and(&mut self) -> Result<Term, EvalError> {
        let mut term = self.parse_less()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_less()?;
            term = Term::And(Box::new(term), Box::new(right));
        }

        Ok(term)
    }

    /// Parses `<`.
    fn parse_less(&mut self) -> Result<Term, EvalError> {
        let mut term = self.parse_add_sub()?;

        while self.check(&Token::Less) {
            self.advance();
            let right = self.parse_add_sub()?;
            term = Term::Less(Box::new(term), Box::new(right));
        }

        Ok(term)
    }

    /// Parses + and - with left associativity.
    fn parse_add_sub(&mut self) -> Result<Term, EvalError> {
        let mut term = self.parse_mul()?;

        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_mul()?;
                    term = Term::Add(Box::new(term), Box::new(right));
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_mul()?;
                    term = Term::Sub(Box::new(term), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(term)
    }

    /// Parses * with higher precedence than + and -.
    fn parse_mul(&mut self) -> Result<Term, EvalError> {
        let mut term = self.parse_application()?;

        while self.check(&Token::Star) {
            self.advance();
            let right = self.parse_application()?;
            term = Term::Mul(Box::new(term), Box::new(right));
        }

        Ok(term)
    }

    /// Parses function application.
    ///
    /// Function application has high precedence, so:
    ///     f x + 1
    /// becomes:
    ///     Add(App(f, x), 1)
    fn parse_application(&mut self) -> Result<Term, EvalError> {
        let mut term = self.parse_unary()?;

        while self.starts_atomic_term() {
            let argument = self.parse_unary()?;
            term = Term::App(Box::new(term), Box::new(argument));
        }

        Ok(term)
    }

    /// Parses unary operators like `not`.
    fn parse_unary(&mut self) -> Result<Term, EvalError> {
        if self.check(&Token::Not) {
            self.advance();
            let inner = self.parse_unary()?;
            Ok(Term::Not(Box::new(inner)))
        } else {
            self.parse_primary()
        }
    }

    /// Checks whether the current token can start an atomic term.
    fn starts_atomic_term(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Number(_))
                | Some(Token::Identifier(_))
                | Some(Token::True)
                | Some(Token::False)
                | Some(Token::LParen)
        )
    }

    /// Parses the smallest term units:
    /// - integers
    /// - booleans
    /// - variables
    /// - parenthesized terms
    fn parse_primary(&mut self) -> Result<Term, EvalError> {
        match self.advance() {
            Some(Token::Number(value)) => Ok(Term::Int(value)),

            Some(Token::True) => Ok(Term::True),
            Some(Token::False) => Ok(Term::False),

            Some(Token::Identifier(name)) => Ok(Term::Var(name)),

            Some(Token::LParen) => {
                let term = self.parse_term()?;
                self.expect(Token::RParen)?;
                Ok(term)
            }

            Some(token) => Err(EvalError::ParseError(format!(
                "unexpected token {:?} while parsing a MiniFun term",
                token
            ))),

            None => Err(EvalError::UnexpectedEndOfInput),
        }
    }

    /// Utility function:
    /// after parsing, there should be no extra tokens left.
    pub fn finish(mut self) -> Result<Term, EvalError> {
        let term = self.parse_term()?;

        if self.peek().is_some() {
            return Err(EvalError::ParseError(format!(
                "unexpected extra token {:?} after the end of the term",
                self.peek()
            )));
        }

        Ok(term)
    }
    
/// Parses a type (e.g., int, bool, int -> int)
fn parse_type(&mut self) -> Result<Type, EvalError> {
    let mut ty = self.parse_atomic_type()?;

    // left associative: int -> bool -> int
    while self.check(&Token::TypeArrow) {
        self.advance();
        let right = self.parse_atomic_type()?;
        ty = Type::Fun(Box::new(ty), Box::new(right));
    }

    Ok(ty)
}

/// Parses atomic types
fn parse_atomic_type(&mut self) -> Result<Type, EvalError> {
    match self.advance() {
        Some(Token::TypeInt) => Ok(Type::Int),

        Some(Token::TypeBool) => Ok(Type::Bool),

        Some(Token::LParen) => {
            let ty = self.parse_type()?;
            self.expect(Token::RParen)?;
            Ok(ty)
        }

        Some(token) => Err(EvalError::ParseError(format!(
            "unexpected token {:?} while parsing a type",
            token
        ))),

        None => Err(EvalError::UnexpectedEndOfInput),
    }
}






}

/// Parses a full MiniFun token list into an AST term.
pub fn parse_tokens(tokens: Vec<Token>) -> Result<Term, EvalError> {
    let parser = Parser::new(tokens);
    parser.finish()
}