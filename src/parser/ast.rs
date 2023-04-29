// src/parser/ast.rs
use crate::parser::lexer::{tokenize, Token};

#[derive(PartialEq, Debug, Clone)]
pub struct Clause {
    pub head: Term,
    pub body: Vec<Term>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Term {
    Atom(String),
    Variable(String),
    Structure {
        functor: String,
        arity: usize,
        args: Vec<Term>,
    },
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse(input: &str) -> ParseResult<Vec<Clause>> {
    let tokens = tokenize(input)?;
    let mut clauses = Vec::new();

    let mut remaining_tokens = &tokens[..];
    while !remaining_tokens.is_empty() {
        let (head, rest) = parse_term(remaining_tokens)?;
        remaining_tokens = rest;

        let body = if let Ok(new_remaining_tokens) = expect_token(Token::If, remaining_tokens) {
            remaining_tokens = new_remaining_tokens;
            let mut body_terms = Vec::new();

            while let Ok((term, new_remaining_tokens)) = parse_term(remaining_tokens) {
                remaining_tokens = new_remaining_tokens;
                body_terms.push(term);

                if let Ok(new_remaining_tokens) = expect_token(Token::And, remaining_tokens) {
                    remaining_tokens = new_remaining_tokens;
                } else {
                    break;
                }
            }

            body_terms
        } else {
            Vec::new()
        };

        remaining_tokens = expect_token(Token::Dot, remaining_tokens)?;
        clauses.push(Clause { head, body });
    }

    Ok(clauses)
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
    fn test_parse_term1() {
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
    
    #[test]
    fn test_expect_token() {
        use super::{expect_token, ParseError, Token};

        let tokens = [
            Token::Atom("hello".to_string()),
            Token::Variable("Var".to_string()),
            Token::LParen,
        ];

        // Test successful case
        let expected = Token::Atom("hello".to_string());
        let remaining_tokens = &tokens[1..];
        match expect_token(expected.clone(), &tokens) {
            Ok(result) => assert_eq!(result, remaining_tokens),
            Err(_) => panic!("expect_token should return Ok in this case"),
        }

        // Test error case
        let expected = Token::Variable("WrongVar".to_string());
        match expect_token(expected, &tokens) {
            Ok(_) => panic!("expect_token should return Err in this case"),
            Err(e) => match e {
                ParseError::UnexpectedToken(token) => {
                    assert_eq!(token, Token::Atom("hello".to_string()))
                }
                _ => panic!("expect_token should return UnexpectedToken error"),
            },
        }
    }

    #[test]
    fn test_parse_term2() {
        use super::{parse_term, Term, Token};

        // Test parsing an atom
        let tokens = [
            Token::Atom("hello".to_string()),
            Token::Variable("Var".to_string()),
            Token::LParen,
        ];
        let expected_term = Term::Atom("hello".to_string());
        let remaining_tokens = &tokens[1..];
        match parse_term(&tokens) {
            Ok((term, rest)) => {
                assert_eq!(term, expected_term);
                assert_eq!(rest, remaining_tokens);
            }
            Err(_) => panic!("parse_term should return Ok in this case"),
        }

        // Test parsing a variable
        let tokens = [
            Token::Variable("Var".to_string()),
            Token::Atom("hello".to_string()),
            Token::RParen,
        ];
        let expected_term = Term::Variable("Var".to_string());
        let remaining_tokens = &tokens[1..];
        match parse_term(&tokens) {
            Ok((term, rest)) => {
                assert_eq!(term, expected_term);
                assert_eq!(rest, remaining_tokens);
            }
            Err(_) => panic!("parse_term should return Ok in this case"),
        }
    }

    #[test]
    fn test_parse_simple_structure() {
        let tokens = vec![
            Token::Atom("parent".to_string()),
            Token::LParen,
            Token::Atom("jim".to_string()),
            Token::RParen,
        ];
        let parse_result = parse_term(&tokens);
        let (term, remaining_tokens) = parse_result.unwrap();
        let expected_term = Term::Structure {
            functor: "parent".to_string(),
            arity: 1,
            args: vec![
                Term::Atom("jim".to_string()),
            ],
        };
        assert_eq!(term, expected_term);
        assert!(remaining_tokens.is_empty());
    }

    #[test]
    fn test_parse_structure_with_two_args() {
        let tokens = vec![
            Token::Atom("parent".to_string()),
            Token::LParen,
            Token::Atom("jim".to_string()),
            Token::Comma,
            Token::Atom("ann".to_string()),
            Token::RParen,
        ];
        let parse_result = parse_term(&tokens);
        let (term, remaining_tokens) = parse_result.unwrap();
        let expected_term = Term::Structure {
            functor: "parent".to_string(),
            arity: 2,
            args: vec![
                Term::Atom("jim".to_string()),
                Term::Atom("ann".to_string()),
            ],
        };
        assert_eq!(term, expected_term);
        assert!(remaining_tokens.is_empty());
    }

    #[test]
    fn test_parse_nested_structure() {
        let tokens = vec![
            Token::Atom("parent".to_string()),
            Token::LParen,
            Token::Atom("jim".to_string()),
            Token::Comma,
            Token::Atom("child".to_string()),
            Token::LParen,
            Token::Atom("ann".to_string()),
            Token::Comma,
            Token::Atom("5".to_string()),
            Token::RParen,
            Token::RParen,
        ];
        let parse_result = parse_term(&tokens);
        let (term, remaining_tokens) = parse_result.unwrap();
        let expected_term = Term::Structure {
            functor: "parent".to_string(),
            arity: 2,
            args: vec![
                Term::Atom("jim".to_string()),
                Term::Structure {
                    functor: "child".to_string(),
                    arity: 2,
                    args: vec![
                        Term::Atom("ann".to_string()),
                        Term::Atom("5".to_string()),
                    ],
                },
            ],
        };
        assert_eq!(term, expected_term);
        assert!(remaining_tokens.is_empty());
    }

    #[test]
    fn test_parse_clause() {
        let input = "likes(john, pizza).";
        let parse_result = parse(input);
        let clauses = parse_result.unwrap();
    
        let expected_clause = Clause {
            head: Term::Structure {
                functor: "likes".to_string(),
                arity: 2,
                args: vec![
                    Term::Atom("john".to_string()),
                    Term::Atom("pizza".to_string()),
                ],
            },
            body: vec![],
        };
        assert_eq!(clauses, vec![expected_clause]);
    }

    #[test]
    fn test_parse_clause_with_body() {
        let input = "likes(john, X) :- likes(X, pizza).";
        let parse_result = parse(input);
        let clauses = parse_result.unwrap();

        let expected_clause = Clause {
            head: Term::Structure {
                functor: "likes".to_string(),
                arity: 2,
                args: vec![
                    Term::Atom("john".to_string()),
                    Term::Variable("X".to_string()),
                ],
            },
            body: vec![
                Term::Structure {
                    functor: "likes".to_string(),
                    arity: 2,
                    args: vec![
                        Term::Variable("X".to_string()),
                        Term::Atom("pizza".to_string()),
                    ],
                },
            ],
        };
        assert_eq!(clauses, vec![expected_clause]);
    }
}
    
    