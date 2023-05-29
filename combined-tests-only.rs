

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
