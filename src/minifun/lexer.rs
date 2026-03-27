use crate::common::error::EvalError;

/// Tokens are the small building blocks produced by the lexer.
/// The parser will consume these tokens and build the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i64),

    // Keywords
    True,
    False,
    If,
    Then,
    Else,
    Let,
    In,
    Fun,
    LetFun,

    // Logical operators
    And,   // &&
    Not,   // ~

    // Type keywords
    TypeInt,    // int
    TypeBool,   // bool

    // Symbols / operators
    Equal,      // =
    Arrow,      // =>
    Colon,      // :
    TypeArrow,  // ->
    LParen,     // (
    RParen,     // )
    Plus,       // +
    Minus,      // -
    Star,       // *
    Less,       // <
}

/// Converts source code text into a list of tokens.
pub fn tokenize(source: &str) -> Result<Vec<Token>, EvalError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let current = chars[i];

        // Ignore spaces, tabs, and newlines.
        if current.is_whitespace() {
            i += 1;
            continue;
        }

        // Read identifiers and keywords.
        if current.is_alphabetic() {
            let start = i;

            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }

            let word: String = chars[start..i].iter().collect();

            let token = match word.as_str() {
                "true" => Token::True,
                "false" => Token::False,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "let" => Token::Let,
                "in" => Token::In,
                "fun" => Token::Fun,
                "letfun" => Token::LetFun,
                "int" => Token::TypeInt,
                "bool" => Token::TypeBool,
                _ => Token::Identifier(word),
            };

            tokens.push(token);
            continue;
        }

        // Read integer literals.
        if current.is_ascii_digit() {
            let start = i;

            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }

            let number_text: String = chars[start..i].iter().collect();
            let value = number_text.parse::<i64>().map_err(|_| {
                EvalError::ParseError(format!("invalid integer literal '{}'", number_text))
            })?;

            tokens.push(Token::Number(value));
            continue;
        }

        // Read symbols and operators.
        match current {
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::Arrow);
                    i += 2;
                } else {
                    tokens.push(Token::Equal);
                    i += 1;
                }
            }
            ':' => {
                tokens.push(Token::Colon);
                i += 1;
            }
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::TypeArrow);
                    i += 2;
                } else {
                    tokens.push(Token::Minus);
                    i += 1;
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token::And);
                    i += 2;
                } else {
                    return Err(EvalError::ParseError(
                        "expected '&' after '&' to form '&&'".to_string(),
                    ));
                }
            }
            '~' => {
                tokens.push(Token::Not);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '*' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '<' => {
                tokens.push(Token::Less);
                i += 1;
            }
            _ => {
                return Err(EvalError::ParseError(format!(
                    "unexpected character '{}'",
                    current
                )));
            }
        }
    }

    Ok(tokens)
}