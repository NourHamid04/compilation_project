use crate::common::error::EvalError;

// Tokens recognized by the MiniImp lexer

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i64),

    // Keywords
    Skip,
    If,
    Then,
    Else,
    While,
    Do,
    True,
    False,
    And,
    Not,

    // Symbols / operators
    Assign,     // :=
    Semicolon,  // ;
    LParen,     // (
    RParen,     // )
    Plus,       // +
    Minus,      // -
    Star,       // *
    Less,       // <
}

// Convert source code into a sequence of tokens
pub fn tokenize(source: &str) -> Result<Vec<Token>, EvalError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
// Scan the input one character at a time
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
                "skip" => Token::Skip,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "while" => Token::While,
                "do" => Token::Do,
                "true" => Token::True,
                "false" => Token::False,
                "and" => Token::And,
                "not" => Token::Not,
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

// Recognize MiniImp symbols and operators
        match current {
            ':' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Assign);
                    i += 2;
                } else {
                    return Err(EvalError::ParseError(
                        "expected '=' after ':' to form ':='".to_string(),
                    ));
                }
            }
            ';' => {
                tokens.push(Token::Semicolon);
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
            '-' => {
                tokens.push(Token::Minus);
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