use crate::{
    ast::Program,
    compile::Compile,
    error::RuntimeError,
    stack::{SmInstruction, StackMachine},
    validate::Validate,
};
use failure::Error;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Write};

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
///
/// The output of this machine is either "ACCEPT" or "REJECT". See the
/// individual run functions to determine where the output destination is.
#[derive(Debug, Serialize)]
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

    pub fn from_json(json: &str) -> Result<Self, Error> {
        let program = serde_json::from_str(&json)?;
        Self::new(program)
    }

    /// Helper function to execute the machine with the given input string and
    /// output destination.
    fn run_with_io<W: Write>(
        &self,
        input: String,
        output: &mut W,
    ) -> Result<(), Error> {
        // Validate each input character
        let illegal_chars: Vec<char> = input
            .chars()
            .filter(|c| !c.is_ascii() || *c == '\x00')
            .collect();

        if illegal_chars.is_empty() {
            let mut machine = StackMachine::new();
            machine.run(input.as_bytes(), output, &self.instructions);
            Ok(())
        } else {
            Err(RuntimeError::IllegalInputCharacters(illegal_chars).into())
        }
    }

    /// Executes this machine on the given input. Uses stdout as the output
    /// stream.
    pub fn run(&self, input: String) -> Result<(), Error> {
        self.run_with_io(input, &mut io::stdout())
    }

    /// Executes this machine on the given input. Returns a byte vector that
    /// contains all of the output from the machine.
    pub fn run_with_output(&self, input: String) -> Result<Vec<u8>, Error> {
        let mut output_buffer = Vec::new();
        self.run_with_io(input, &mut output_buffer)?;
        Ok(output_buffer)
    }
}

impl Display for TuringMachine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for instruction in &self.instructions {
            writeln!(f, "{}", instruction)?;
        }
        Ok(())
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

    fn assert_tm(
        tm: &TuringMachine,
        input: &str,
        should_accept: bool,
    ) -> Result<(), Error> {
        // We have to reverse the input cause TMing is hard
        let output = tm.run_with_output(input.chars().rev().collect())?;
        let output_string = String::from_utf8(output).unwrap();
        let expected_output = if should_accept { "ACCEPT" } else { "REJECT" };
        assert!(
            output_string.ends_with(expected_output),
            "TM returned wrong output: {}",
            output_string
        );
        Ok(())
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
    fn test_simple_machine() -> Result<(), Error> {
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
        assert_tm(&tm, "foo", true)
    }
}
