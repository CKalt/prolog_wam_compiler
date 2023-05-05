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
    List(Vec<Term>),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse(input: &str) -> ParseResult<Vec<Clause>> {
    let tokens = tokenize(input)?;
    println!("Tokens: {:?}", tokens); // Debug output added here
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

        println!("Head: {:?}, Body: {:?}", head, body); // Debug output added here

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

fn parse_term<'a>(tokens: &'a [Token]) -> ParseResult<(Term, &'a [Token])> {
    println!("parse_term: {:?}", tokens);
    let (token, rest) = expect_any_token(tokens)?;
    match token {
        Token::Atom(atom) => {
            let (next_token, next_rest) = expect_any_token(rest)?;
            if next_token == Token::LParen {
                let mut args = Vec::new();
                let mut remaining_tokens = next_rest;
                loop {
                    let (arg, new_remaining_tokens) = parse_term(remaining_tokens)?;
                    args.push(arg);
                    remaining_tokens = new_remaining_tokens;

                    let (next_token, new_remaining_tokens) = expect_any_token(remaining_tokens)?;
                    if next_token == Token::RParen {
                        return Ok((Term::Structure {
                            functor: atom,
                            arity: args.len(),
                            args,
                        }, new_remaining_tokens));
                    } else if next_token != Token::Comma {
                        return Err(ParseError::UnexpectedToken(next_token));
                    } else {
                        remaining_tokens = new_remaining_tokens;
                    }
                }
            } else {
                Ok((Term::Atom(atom), rest))
            }
        }
        Token::Variable(variable) => Ok((Term::Variable(variable), rest)),
        Token::LParen => {
            let (term, rest) = parse_term(rest)?;
            let rest = expect_token(Token::RParen, rest)?;
            Ok((term, rest))
        }
        Token::LBracket => {
            let (list_elements, rest) = parse_list_elements(rest)?;
            let rest = expect_token(Token::RBracket, rest)?;
            Ok((Term::List(list_elements), rest))
        }
        Token::Number(number) => Ok((Term::Atom(number.to_string()), rest)),
        _ => Err(ParseError::UnexpectedToken(token)),
    }
}

fn parse_list_elements<'a>(tokens: &'a [Token]) -> ParseResult<(Vec<Term>, &'a [Token])> {
    let mut elements = Vec::new();
    let mut remaining_tokens = tokens;

    while let Ok((term, new_remaining_tokens)) = parse_term(remaining_tokens) {
        elements.push(term);
        remaining_tokens = new_remaining_tokens;

        if let Ok(new_remaining_tokens) = expect_token(Token::Comma, remaining_tokens) {
            remaining_tokens = new_remaining_tokens;
        } else {
            break;
        }
    }
    println!("Parsed list elements: {:?}, remaining tokens: {:?}", elements, remaining_tokens);

    Ok((elements, remaining_tokens))
}

fn expect_any_token<'a>(tokens: &'a [Token]) -> ParseResult<(Token, &'a [Token])> {
    if tokens.is_empty() {
        Err(ParseError::UnexpectedEndOfInput)
    } else {
        Ok((tokens[0].clone(), &tokens[1..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Token;

    #[test]
    fn test_parse_term1() {
        println!("Starting test_parse_term1");
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
        println!("Ending test_parse_term1");
    } 
    
    #[test]
    fn test_expect_token() {
        println!("Starting test_expect_token");

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
        println!("Ending test_expect_token");
    }

    #[test]
    fn test_parse_term2() {
        println!("Starting test_parse_term2");
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
        println!("Ending test_parse_term2");
    }

    #[test]
    fn test_parse_simple_structure() {
        println!("Starting test_parse_simple_structure");

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
        println!("Ending test_parse_simple_structure");
    }

    #[test]
    fn test_parse_structure_with_two_args() {
        println!("Starting test_parse_structure_with_two_args");
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
        println!("Ending test_parse_structure_with_two_args");
    }

    #[test]
    fn test_parse_nested_structure() {
        println!("Starting test_parse_nested_structure");
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
        println!("Ending test_parse_nested_structure");
    }

    #[test]
    fn test_parse_clause() {
        println!("Starting test_parse_clause");

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
        println!("Ending test_parse_clause");
    }

    #[test]
    fn test_parse_clause_with_body() {
        println!("Starting test_parse_clause_with_body");
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
        println!("Ending test_parse_clause_with_body");
    }

    #[test]
    fn test_parse_list() {
        println!("Starting test_parse_list");
        let input = "[1, 2, 3]";
        let tokens = tokenize(input).unwrap();
        println!("Tokens: {:?}", tokens);
        let (term, remaining_tokens) = parse_term(&tokens).unwrap();
        println!("Parsed term: {:?}", term);
        println!("Remaining tokens: {:?}", remaining_tokens);
    
        assert!(matches!(term, Term::List(_)));
        assert_eq!(remaining_tokens.len(), 0);
        println!("Ending test_parse_list");
    }
}
    
    