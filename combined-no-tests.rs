

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


---------src/wam/mod.rs---------


// src/wam/mod.rs
pub mod data_structures;

pub use data_structures::{WamEmulator, Term, HeapCell};
