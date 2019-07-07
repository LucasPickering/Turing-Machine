use crate::{
    ast::Program,
    compile::Compile,
    stack::{SmInstruction, StackMachine},
};
use std::io;

/// A Turing machine built entirely on Rocketlang's stack machine. This proves
/// that Rocketlang is Turing-complete.
///
/// The alphabet for this is the characters [0, ALPHABET_SIZE), where 0 is the
/// empty char (and therefore is _not_ valid input).
///
/// This machine should not be exposed externally, because it assumes that the
/// input states have been validated.
///
/// This has the external API of a TM, but internally only uses the two-variable
/// stack machine from Rocketlang.
pub struct TuringMachine {
    instructions: Vec<SmInstruction>,
}

impl TuringMachine {
    /// Constructs a new Turing machine with the given states. This assumes that
    /// all necessary validation has been run on the states. This includes
    /// ensuring that the IDs are sequential, the initial state is in the array,
    /// etc.
    pub fn new(program: Program) -> Self {
        // TODO validation
        Self {
            instructions: program.compile(),
        }
    }

    pub fn run(&self, input: String) {
        // TODO Input validation
        let mut machine = StackMachine::new(input.as_bytes(), io::stdout());
        machine.run(&self.instructions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::State;

    #[test]
    fn test_noop() {
        let tm = TuringMachine::new(vec![State {
            id: 0,
            initial: true,
            accepting: true,
            transitions: vec![],
        }]);
        assert!(!tm.instructions.is_empty());
        tm.run("\x00".to_owned());
    }
}
