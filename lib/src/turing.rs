use crate::{
    ast::Program,
    compile::Compile,
    error::RuntimeError,
    stack::{SmInstruction, StackMachine},
    validate::Validate,
};
use failure::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

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
#[derive(Debug)]
pub struct TuringMachine {
    instructions: Vec<SmInstruction>,
}

impl TuringMachine {
    /// Constructs a new Turing machine with the given states. This assumes that
    /// all necessary validation has been run on the states. This includes
    /// ensuring that the IDs are sequential, the initial state is in the array,
    /// etc.
    pub fn new(program: Program) -> Result<Self, Error> {
        Ok(Self {
            instructions: program.validate_into(&())?.compile(),
        })
    }

    pub fn from_file(path: &PathBuf) -> Result<Self, Error> {
        let contents = fs::read_to_string(path)?;
        let program = serde_json::from_str(&contents)?;
        Self::new(program)
    }

    pub fn run(&self, input: String) -> Result<(), Error> {
        // Validate each input character
        let illegal_chars: Vec<char> = input
            .chars()
            .filter(|c| !c.is_ascii() || *c == '\x00')
            .collect();

        if illegal_chars.is_empty() {
            let mut machine = StackMachine::new(input.as_bytes(), io::stdout());
            machine.run(&self.instructions);
            Ok(())
        } else {
            Err(RuntimeError::IllegalInputCharacters(illegal_chars).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{State, TapeInstruction, Transition};
    use crate::utils::assert_error;

    /// Converts the given character to a byte by taking the lowest 8 bits. If any
    /// upper bits are non-zero, they are lost and the character is mutated.
    fn char_to_u8(c: char) -> u8 {
        c as u8
    }

    #[test]
    fn test_validation_runs() {
        // Just make sure Program validation gets called
        let tm_result = TuringMachine::new(Program {
            states: vec![State {
                id: 0,
                initial: true,
                accepting: true,
                transitions: vec![],
            }],
        });
        assert_error("Invalid state ID: 0", tm_result);
    }

    #[test]
    fn test_null_in_input_error() {
        let tm = TuringMachine::new(Program {
            states: vec![State {
                id: 1,
                initial: true,
                accepting: true,
                transitions: vec![],
            }],
        })
        .unwrap();
        assert_error("Illegal character: \x00", tm.run("\x00".into()));
    }

    #[test]
    fn test_non_ascii_in_input_error() {
        let tm = TuringMachine::new(Program {
            states: vec![State {
                id: 1,
                initial: true,
                accepting: true,
                transitions: vec![],
            }],
        })
        .unwrap();
        assert_error("Illegal character: \u{80}", tm.run("\u{80}".into()));
    }

    #[test]
    fn test_simple_machine() {
        // Machine matches the string "foo"
        let tm = TuringMachine::new(Program {
            states: vec![
                State {
                    id: 1,
                    initial: true,
                    accepting: false,
                    transitions: vec![Transition {
                        match_char: char_to_u8('f'),
                        tape_instruction: TapeInstruction::Right,
                        next_state: 2,
                    }],
                },
                State {
                    id: 2,
                    initial: false,
                    accepting: false,
                    transitions: vec![Transition {
                        match_char: char_to_u8('o'),
                        tape_instruction: TapeInstruction::Right,
                        next_state: 3,
                    }],
                },
                State {
                    id: 3,
                    initial: false,
                    accepting: false,
                    transitions: vec![Transition {
                        match_char: char_to_u8('o'),
                        tape_instruction: TapeInstruction::Right,
                        next_state: 4,
                    }],
                },
                State {
                    id: 4,
                    initial: false,
                    accepting: true,
                    transitions: vec![],
                },
            ],
        })
        .unwrap();
        assert!(tm.run("foo".into()).is_ok());
    }
}
