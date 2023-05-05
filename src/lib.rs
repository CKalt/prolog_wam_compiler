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
