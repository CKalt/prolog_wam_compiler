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
