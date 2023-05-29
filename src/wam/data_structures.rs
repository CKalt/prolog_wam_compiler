// src/wam/data_structures.rs
pub enum Term {
    Atom(String),
    Compound(String, Vec<Term>), // Changed from Compound(String, Vec<Term>)
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
            Term::Compound(name, terms) => {
                let mut indexes = Vec::new();
                for term in terms {
                    let index = self.push_term(term);
                    indexes.push(index);
                }
                let index = self.heap.len();
                self.heap.push(HeapCell::Structure(name.clone(), indexes));
                index
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
    fn test_push_term() {
        let mut wam = WamEmulator::new();

        // Test Atom
        let index_atom = wam.push_term(&Term::Atom("atom".into()));
        assert_eq!(wam.get_heap_cell(index_atom), Some(&HeapCell::Constant("atom".into())));

        // Test Variable
        let index_var = wam.push_term(&Term::Variable);
        assert_eq!(wam.get_heap_cell(index_var), Some(&HeapCell::Reference(index_var)));

        // Test Compound
        let index_compound = wam.push_term(&Term::Compound("compound".into(), vec![Term::Atom("child".into())]));
        // Update this line
        assert_eq!(wam.get_heap_cell(index_compound), Some(&HeapCell::Structure("compound".into(), vec![index_var + 1])));
    }
}

#[test]
fn test_nested_compound_terms() {
    let mut wam = WamEmulator::new();
    let index_compound = wam.push_term(&Term::Compound("compound1".into(), vec![
        Term::Compound("compound2".into(), vec![
            Term::Atom("child".into())
        ])
    ]));
    assert_eq!(wam.get_heap_cell(index_compound), Some(&HeapCell::Structure("compound1".into(), vec![1])));
    assert_eq!(wam.get_heap_cell(1), Some(&HeapCell::Structure("compound2".into(), vec![0])));
    assert_eq!(wam.get_heap_cell(0), Some(&HeapCell::Constant("child".into())));
}

#[test]
fn test_deep_nested_compound_terms() {
    let mut wam = WamEmulator::new();
    let index_compound = wam.push_term(&Term::Compound("compound1".into(), vec![
        Term::Compound("compound2".into(), vec![
            Term::Compound("compound3".into(), vec![
                Term::Compound("compound4".into(), vec![
                    Term::Atom("child".into())
                ])
            ])
        ])
    ]));
    
    assert_eq!(wam.get_heap_cell(index_compound), Some(&HeapCell::Structure("compound1".into(), vec![3])));
    assert_eq!(wam.get_heap_cell(3), Some(&HeapCell::Structure("compound2".into(), vec![2])));
    assert_eq!(wam.get_heap_cell(2), Some(&HeapCell::Structure("compound3".into(), vec![1])));
    assert_eq!(wam.get_heap_cell(1), Some(&HeapCell::Structure("compound4".into(), vec![0])));
    assert_eq!(wam.get_heap_cell(0), Some(&HeapCell::Constant("child".into())));
}

#[test]
fn test_multiple_variables_in_compound() {
    let mut wam = WamEmulator::new();
    let index_compound = wam.push_term(&Term::Compound("compound".into(), vec![
        Term::Variable, 
        Term::Variable, 
        Term::Variable
    ]));

    // We assume that index of the first Variable in compound is 0.
    assert_eq!(wam.get_heap_cell(0), Some(&HeapCell::Reference(0)));
    // The index of the second Variable in compound is 1.
    assert_eq!(wam.get_heap_cell(1), Some(&HeapCell::Reference(1)));
    // The index of the third Variable in compound is 2.
    assert_eq!(wam.get_heap_cell(2), Some(&HeapCell::Reference(2)));
    // The index of the Compound is 3.
    assert_eq!(wam.get_heap_cell(index_compound), Some(&HeapCell::Structure("compound".into(), vec![0, 1, 2])));
}

#[test]
fn test_get_invalid_index() {
    let mut wam = WamEmulator::new();
    wam.push_term(&Term::Atom("atom".into()));

    // get_heap_cell should return None for indexes that don't exist in the heap.
    assert_eq!(wam.get_heap_cell(999), None);
}
