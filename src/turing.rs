use crate::{
    ast::Program,
    compile::Compile,
    validate::Validate,
    error::{CompilerError, CompilerResult},
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
    pub fn new(program: Program) -> CompilerResult<Self> {
        Ok(Self {
            instructions: program.validate_into(&())?.compile(),
        })
    }

    pub fn run(&self, input: String) -> CompilerResult<()> {
        // Validate each input character
        let errors: Vec<CompilerError> = input
            .chars()
            .filter(|c| !c.is_ascii() || *c == '\x00')
            .map(CompilerError::InvalidCharacter)
            .collect();

        if errors.is_empty() {
            let mut machine = StackMachine::new(input.as_bytes(), io::stdout());
            machine.run(&self.instructions);
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::State;
    use std::fmt::Debug;

    fn assert_error<T: Debug>(msg: &str, result: CompilerResult<T>) {
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .iter()
            .any(|err| err.to_string().contains(msg)));
    }

    #[test]
    fn test_null_in_input() {
        let tm_result = TuringMachine::new(Program{states:vec![State {
            id: 0,
            initial: true,
            accepting: true,
            transitions: vec![],
        }]});
        assert!(tm_result.is_ok());
        assert_error(
            "Invalid character: \x00",
            tm_result.unwrap().run("\x00".into()),
        );
    }
}
