

---------src/lib.rs---------


// src/lib.rs
pub mod wam;
pub mod parser;

pub use wam::{WamEmulator, Term, HeapCell};
pub use parser::lexer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_whitespace() {
        println!("Starting test_skip_whitespace");
        let input = "   some text";
        let result = lexer::skip_whitespace(input);
        assert_eq!(result, "some text");
        println!("Ending test_skip_whitespace");
    }
}


---------src/main.rs---------


// src/main.rs
use prolog_wam_compiler::{WamEmulator, Term};

fn main() {
    let term = Term::Atom("example".to_string());

    let mut emulator = WamEmulator::new();
    let index = emulator.push_term(&term);

    println!("Term pushed to heap at index: {}", index);
}


---------src/parser/ast.rs---------


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

                // Change this part
                if let Ok(new_remaining_tokens) = expect_token(Token::And, remaining_tokens)
                    .or(expect_token(Token::Comma, remaining_tokens))
                {
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

    #[test]
    fn test_parse_clause_with_double_body() {
        println!("Starting test_parse_clause_with_double_body");
        let input = "parents(X, Y) :- father(X, Y), mother(X, Y).";
        let parse_result = parse(input);
        let clauses = parse_result.unwrap();

        let expected_clause = Clause {
            head: Term::Structure {
                functor: "parents".to_string(),
                arity: 2,
                args: vec![
                    Term::Variable("X".to_string()),
                    Term::Variable("Y".to_string()),
                ],
            },
            body: vec![
                Term::Structure {
                    functor: "father".to_string(),
                    arity: 2,
                    args: vec![
                        Term::Variable("X".to_string()),
                        Term::Variable("Y".to_string()),
                    ],
                },
                Term::Structure {
                    functor: "mother".to_string(),
                    arity: 2,
                    args: vec![
                        Term::Variable("X".to_string()),
                        Term::Variable("Y".to_string()),
                    ],
                },
            ],
        };
        assert_eq!(clauses, vec![expected_clause]);
        println!("Ending test_parse_clause_with_double_body");
    }
}
    

---------src/parser/lexer.rs---------


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
                if first_char == 'i' && iter.peek() == Some(&'s') {
                    iter.next(); // skip the 's'
                    tokens.push(Token::Is);
                } else {
                    tokens.push(parse_atom_or_variable(first_char, &mut iter)?);
                }
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
            '-' => {
                iter.next();
                match iter.peek() {
                    Some(c) if c.is_digit(10) => tokens.push(parse_integer('-', &mut iter)?),
                    _ => tokens.push(Token::Minus),
                }
            }
            '*' => {
                iter.next();
                tokens.push(Token::Multiply);
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
    Minus,
    Multiply,
    Divide,
}


#[derive(Debug)]
pub enum LexerError {
    UnexpectedChar(char),
    InvalidInteger(String),
    UnexpectedEndOfInput
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

    #[test]
    fn test_tokenize_with_comment_plus_and_is() {
        let input = r#"
            % Let's test whether an is expression is parsable
            plus(A, B, C) :- C is A + B.
        "#;

        let expected_tokens = vec![
            Token::Atom("plus".to_string()),
            Token::LParen,
            Token::Variable("A".to_string()),
            Token::Comma,
            Token::Variable("B".to_string()),
            Token::Comma,
            Token::Variable("C".to_string()),
            Token::RParen,
            Token::If,
            Token::Variable("C".to_string()),
            Token::Is,
            Token::Variable("A".to_string()),
            Token::Plus,
            Token::Variable("B".to_string()),
            Token::Dot,
        ];

        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens, expected_tokens);
    }
        
    #[test]
    fn tokenize_minus() {
        let result = tokenize("-");
        assert_eq!(result.unwrap(), vec![Token::Minus]);
    }

    #[test]
    fn tokenize_multiply() {
        let result = tokenize("*");
        assert_eq!(result.unwrap(), vec![Token::Multiply]);
    }

    #[test]
    fn tokenize_expression() {
        let result = tokenize("1 + 1 - 2 * 3");
        assert_eq!(result.unwrap(), vec![
            Token::Number(1),
            Token::Plus,
            Token::Number(1),
            Token::Minus,
            Token::Number(2),
            Token::Multiply,
            Token::Number(3),
        ]);
    }    
}

---------src/parser/mod.rs---------


// mod.rs
pub mod ast;
pub mod lexer;


---------src/wam/data_structures.rs---------


// src/wam/data_structures.rs
pub enum Term {
    Atom(String),
    Compound(String, Vec<Term>),
    Variable,
}

#[derive(Debug, PartialEq)]
pub enum HeapCell {
    Reference(usize),
    Structure(String, Vec<usize>),
    Constant(String),
}

pub struct WamEmulator {
    heap: Vec<HeapCell>,
}

impl WamEmulator {
    pub fn new() -> Self {
        Self { heap: Vec::new() }
    }

    pub fn push_term(&mut self, term: &Term) -> usize {
        match term {
            Term::Atom(name) => {
                let index = self.heap.len();
                self.heap.push(HeapCell::Constant(name.clone()));
                index
            }
            Term::Compound(_, _) => {
                // Implement compound term storage here
                unimplemented!()
            }
            Term::Variable => {
                let index = self.heap.len();
                self.heap.push(HeapCell::Reference(index));
                index
            }
        }
    }

    pub fn get_heap_cell(&self, index: usize) -> Option<&HeapCell> {
        self.heap.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_atom() {
        println!("Starting test_push_atom");
        let term = Term::Atom("example".to_string());
        let mut emulator = WamEmulator::new();
        let index = emulator.push_term(&term);
        let cell = emulator.get_heap_cell(index).unwrap();

        assert_eq!(cell, &HeapCell::Constant("example".to_string()));
        println!("Ending test_push_atom");
    }
}


---------src/wam/mod.rs---------


// src/wam/mod.rs
pub mod data_structures;

pub use data_structures::{WamEmulator, Term, HeapCell};
