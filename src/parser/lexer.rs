use std::iter::Peekable;

fn parse_atom_or_variable<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Token, LexerError> {
    let mut name = String::new();

    while let Some(&c) = iter.peek() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                name.push(c);
                iter.next();
            }
            _ => break,
        }
    }

    let token = if name.chars().next().unwrap().is_uppercase() || name.starts_with('_') {
        Token::Variable(name)
    } else {
        Token::Atom(name)
    };

    Ok(token)
}

fn parse_integer<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Token, LexerError> {
    let mut number = String::new();

    while let Some(&c) = iter.peek() {
        match c {
            '0'..='9' => {
                number.push(c);
                iter.next();
            }
            _ => break,
        }
    }

    let value = number
        .parse::<i64>()
        .map_err(|_| LexerError::InvalidInteger(number))?;

    Ok(Token::Integer(value))
}

pub fn skip_whitespace(input: &str) -> &str {
    input.trim_start()
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(&c) = iter.peek() {
        match c {
            'A'..='Z' | 'a'..='z' | '_' => {
                // tokens.push(parse_atom_or_variable::<I>(&mut iter)?);  // fails with I undefined.
                tokens.push(parse_atom_or_variable(&mut iter)?);
            }
            '0'..='9' => {
                tokens.push(parse_integer(&mut iter)?);
            }
            '(' => {
                iter.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                iter.next();
                tokens.push(Token::RParen);
            }
            ',' => {
                iter.next();
                tokens.push(Token::Comma);
            }
            '.' => {
                iter.next();
                tokens.push(Token::Dot);
            }
            ':' => {
                iter.next();
                if let Some(&'-') = iter.peek() {
                    iter.next();
                    tokens.push(Token::If);
                } else {
                    return Err(LexerError::UnexpectedChar(c));
                }
            }
            ';' => {
                iter.next();
                tokens.push(Token::And);
            }
            _ => {
                if c.is_whitespace() {
                    iter.next();
                } else {
                    return Err(LexerError::UnexpectedChar(c));
                }
            }
        }
    }

    Ok(tokens)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Atom(String),
    Variable(String),
    Integer(i64),
    LParen,
    RParen,
    Comma,
    Dot,
    If,
    And,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedChar(char),
    InvalidInteger(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_enum() {
        let atom = Token::Atom("likes".to_string());
        let variable = Token::Variable("X".to_string());
        let left_paren = Token::LParen;
        let right_paren = Token::RParen;
        let comma = Token::Comma;
        let dot = Token::Dot;

        assert_eq!(atom, Token::Atom("likes".to_string()));
        assert_eq!(variable, Token::Variable("X".to_string()));
        assert_eq!(left_paren, Token::LParen);
        assert_eq!(right_paren, Token::RParen);
        assert_eq!(comma, Token::Comma);
        assert_eq!(dot, Token::Dot);
    }

    #[test]
    fn test_tokenize_atoms_and_variables() {
        let input = "likes(X, food).";
        let expected_tokens = vec![
            Token::Atom("likes".to_string()),
            Token::LParen,
            Token::Variable("X".to_string()),
            Token::Comma,
            Token::Atom("food".to_string()),
            Token::RParen,
            Token::Dot,
        ];
    
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, expected_tokens);
    }
}
