// src/parser/lexer.rs
use std::iter::Peekable;

fn parse_atom_or_variable<I: Iterator<Item = char>>(first_char: char, iter: &mut Peekable<I>) -> Result<Token, LexerError> {
    let mut name = String::new();
    name.push(first_char);

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

fn parse_integer<I: Iterator<Item = char>>(first_digit: char, iter: &mut Peekable<I>) -> Result<Token, LexerError> {
    let mut value = String::new();
    value.push(first_digit);

    while let Some(&c) = iter.peek() {
        if c.is_digit(10) {
            value.push(c);
            iter.next();
        } else {
            break;
        }
    }

    if value.is_empty() {
        return Err(LexerError::InvalidInteger(value));
    }

    value
        .parse::<i64>()
        .map(Token::Number)
        .map_err(|_| LexerError::InvalidInteger(value))
}

pub fn skip_whitespace(input: &str) -> &str {
    input.trim_start()
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(&c) = iter.peek() {
        println!("DEBUG: Current character: {:?}", c);
    
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                iter.next(); // skip whitespace
            }
            ',' => {
                iter.next();
                tokens.push(Token::Comma);
                while let Some(&c) = iter.peek() {
                    if c == ' ' || c == ',' {
                        iter.next();
                    } else {
                        break;
                    }
                }
            }
            'A'..='Z' | 'a'..='z' | '_' => {
                let first_char = iter.next().unwrap();
                tokens.push(parse_atom_or_variable(first_char, &mut iter)?);
            }
            '0'..='9' => {
                let first_digit = iter.next().unwrap();
                tokens.push(parse_integer(first_digit, &mut iter)?);
            }
            '(' => {
                iter.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                iter.next();
                tokens.push(Token::RParen);
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
            '[' => {
                iter.next();
                tokens.push(Token::LBracket);
            }
            ']' => {
                iter.next();
                tokens.push(Token::RBracket);
            }
            '%' => {
                iter.next(); // skip the '%'
                while let Some(&c) = iter.peek() {
                    if c != '\n' {
                        iter.next(); // skip non-newline characters in the comment
                    } else {
                        break; // break out of the loop when reaching a newline
                    }
                }
            }
            '/' => {
                iter.next(); // skip the '/'
                if let Some(&'*') = iter.peek() {
                    iter.next(); // skip the '*'
                    let mut comment_level = 1; // track nested multi-line comments
            
                    while comment_level > 0 {
                        if let Some(c) = iter.next() {
                            match c {
                                '*' => {
                                    if let Some(&'/') = iter.peek() {
                                        iter.next(); // skip the '/'
                                        comment_level -= 1;
                                    }
                                }
                                '/' => {
                                    if let Some(&'*') = iter.peek() {
                                        iter.next(); // skip the '*'
                                        comment_level += 1;
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            return Err(LexerError::UnexpectedChar(c));
                        }
                    }
                } else {
                    return Err(LexerError::UnexpectedChar(c));
                }
            }
            '+' => {
                iter.next();
                tokens.push(Token::Plus);
            }
            _ => {
                iter.next();
                if c.is_whitespace() {
                } else {
                    return Err(LexerError::UnexpectedChar(c));
                }
            }
        }
    }

    println!("DEBUG: Final tokens: {:?}", tokens);
    Ok(tokens)
}

pub fn is_valid_atom(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    let first_char = input.chars().next().unwrap();
    if !first_char.is_lowercase() {
        return false;
    }

    input.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Atom(String),
    Variable(String),
    Number(i64),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    If,
    And,
    Is,
    Plus,
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
        println!("Starting test_token_enum");
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
        println!("Ending test_token_enum");
    }

    #[test]
    fn test_tokenize_atoms_and_variables() {
        println!("Starting test_tokenize_atoms_and_variables");
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
        println!("Ending test_tokenize_atoms_and_variables");
    }

    #[test]
    fn test_valid_atom() {
        assert!(is_valid_atom("example_atom"));
        assert!(is_valid_atom("atom_with_underscore"));
        assert!(!is_valid_atom("AtomWithUppercase"));
        assert!(!is_valid_atom("atom with spaces"));
        assert!(!is_valid_atom("123_invalid_start"));
        assert!(!is_valid_atom(""));
    }

    #[test]
    fn test_tokenize_with_comments() {
        let input = r#"
            % A comment
            % Another comment
            a, % Inline comment
            b.
        "#;

        let expected_tokens = vec![
            Token::Atom("a".to_string()),
            Token::Comma,
            Token::Atom("b".to_string()),
            Token::Dot,
        ];

        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens, expected_tokens);
    }
}