// src/parser/ast.rs
use crate::parser::lexer::{tokenize, Token};

#[derive(Debug, PartialEq)] // Added Debug derive for the Term enum
pub enum Term {
    Atom(String),
    Variable(String),
    Structure {
        functor: String,
        arity: usize,
        args: Vec<Term>,
    },
}

pub struct Clause {
    pub head: Term,
    pub body: Vec<Term>,
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse(input: &str) -> ParseResult<Vec<Clause>> {
    let _tokens = tokenize(input)?;
    // TODO: Implement parsing logic
    Ok(vec![]) // Placeholder until the parsing logic is implemented
}

#[derive(Debug)]
pub enum ParseError {
    LexerError(crate::parser::lexer::LexerError),
    UnexpectedToken(Token),
    UnexpectedEndOfInput,
    InvalidToken,
    // Other error variants will be added as needed
}

// This allows us to convert LexerError into ParseError
impl From<crate::parser::lexer::LexerError> for ParseError {
    fn from(error: crate::parser::lexer::LexerError) -> Self {
        ParseError::LexerError(error)
    }
}

#[allow(dead_code)]
fn expect_token<'a>(expected: Token, tokens: &'a [Token]) -> ParseResult<&'a [Token]> {
    if let Some(token) = tokens.first() {
        if *token == expected {
            Ok(&tokens[1..])
        } else {
            Err(ParseError::UnexpectedToken(token.clone()))
        }
    } else {
        Err(ParseError::UnexpectedEndOfInput)
    }
}

#[allow(dead_code)]
fn parse_term<'a>(tokens: &'a [Token]) -> ParseResult<(Term, &'a [Token])> {
    match tokens.first() {
        Some(Token::Atom(functor)) => {
            let tokens = &tokens[1..];
            if let Ok(tokens) = expect_token(Token::LParen, tokens) {
                parse_structure(functor, tokens)
            } else {
                Ok((Term::Atom(functor.clone()), tokens))
            }
        }
        Some(Token::Variable(name)) => {
            let term = Term::Variable(name.clone());
            Ok((term, &tokens[1..]))
        }
        _ => Err(ParseError::InvalidToken),
    }
}

#[allow(dead_code)]
fn parse_structure<'a>(functor: &str, tokens: &'a [Token]) -> ParseResult<(Term, &'a [Token])> {
    let mut args = Vec::new();
    let mut tokens = tokens;
    loop {
        let (arg, remaining_tokens) = parse_term(tokens)?;
        args.push(arg);
        tokens = remaining_tokens;

        match tokens.first() {
            Some(Token::Comma) => tokens = &tokens[1..],
            Some(Token::RParen) => {
                tokens = &tokens[1..];
                break;
            }
            _ => return Err(ParseError::InvalidToken),
        }
    }
    let term = Term::Structure {
        functor: functor.to_string(),
        arity: args.len(),
        args,
    };
    Ok((term, tokens))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Token;

    #[test]
    fn test_parse_term() {
        let tokens = vec![
            Token::Atom("father".to_string()),
            Token::LParen,
            Token::Atom("john".to_string()),
            Token::Comma,
            Token::Atom("jim".to_string()),
            Token::RParen,
        ];
        let parse_result = parse_term(&tokens);
        let (term, remaining_tokens) = parse_result.unwrap();
        let expected_term = Term::Structure {
            functor: "father".to_string(),
            arity: 2,
            args: vec![
                Term::Atom("john".to_string()),
                Term::Atom("jim".to_string()),
            ],
        };
        assert_eq!(term, expected_term);
        assert!(remaining_tokens.is_empty());
    }
}
