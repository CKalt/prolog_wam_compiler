pub fn skip_whitespace(input: &str) -> &str {
    input.trim_start()
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut input_iter = input.chars().peekable();

    while let Some(&c) = input_iter.peek() {
        if c.is_whitespace() {
            input_iter.next(); // Consume the whitespace character
            continue; // Continue to the next iteration of the loop
        }

        match c {
            'A'..='Z' | 'a'..='z' => {
                let name = read_name(&mut input_iter);
                if c.is_uppercase() {
                    tokens.push(Token::Variable(name));
                } else {
                    tokens.push(Token::Atom(name));
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                input_iter.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                input_iter.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                input_iter.next();
            }
            '.' => {
                tokens.push(Token::Dot);
                input_iter.next();
            }
            _ => return Err(LexerError::UnexpectedChar(c)),
        }
    }

    Ok(tokens)
}


fn read_name<I: Iterator<Item = char>>(iter: &mut std::iter::Peekable<I>) -> String {
    let mut name = String::new();
    while let Some(&c) = iter.peek() {
        if c.is_alphanumeric() {
            name.push(c);
            iter.next();
        } else {
            break;
        }
    }
    name
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Atom(String),
    Variable(String),
    LParen,
    RParen,
    Comma,
    Dot,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedChar(char),
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
