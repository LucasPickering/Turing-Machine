use crate::{
    ast::{Program, BLANK_CHAR},
    compile::Compile,
    error::RuntimeError,
    stack::{SmInstruction, StackMachine},
    validate::Validate,
};
use ascii::AsciiString;
use failure::Error;
use serde::Serialize;
use std::{
    fmt::{self, Display, Formatter},
    io::{self, Write},
    str::FromStr,
};

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

    /// Helper function to execute the machine with the given input string and
    /// output destination.
    fn run_with_io<W: Write>(
        &self,
        input: &str,
        output: &mut W,
    ) -> Result<(), Error> {
        let ascii_str = AsciiString::from_str(&input)?;

        for c in ascii_str.chars() {
            if *c == BLANK_CHAR {
                return Err(RuntimeError::BlankCharInInput.into());
            }
        }

        let mut machine = StackMachine::new();
        machine.run(ascii_str.as_bytes(), output, &self.instructions);
        Ok(())
    }

    /// Executes this machine on the given input. Uses stdout as the output
    /// stream.
    pub fn run(&self, input: &str) -> Result<(), Error> {
        self.run_with_io(input, &mut io::stdout())
    }

    /// Executes this machine on the given input. Returns a byte vector that
    /// contains all of the output from the machine.
    pub fn run_with_output(&self, input: &str) -> Result<Vec<u8>, Error> {
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
    use crate::{
        ast::{State, TapeInstruction, Transition},
        utils::assert_error,
    };
    use ascii::AsciiChar;

    fn assert_tm(
        tm: &TuringMachine,
        input: &str,
        should_accept: bool,
    ) -> Result<(), Error> {
        // We have to reverse the input cause TMing is hard
        let output =
            tm.run_with_output(&input.chars().rev().collect::<String>())?;
        let output_string = AsciiString::from_ascii(output).unwrap();
        let expected_output = if should_accept { "ACCEPT" } else { "REJECT" };
        assert!(
            output_string.trim().as_str().ends_with(expected_output),
            "TM returned wrong output. Expected \"{}\", got:\n{}",
            expected_output,
            output_string,
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
    fn test_blank_in_input_error() {
        let tm = TuringMachine::new(Program {
            states: vec![State {
                id: 1,
                initial: true,
                accepting: true,
                transitions: vec![],
            }],
        })
        .unwrap();
        assert_error("Blank char in input", tm.run("\x00"));
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
        assert_error("the byte at index 0 is not ASCII", tm.run("\u{80}"));
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
                        match_char: AsciiChar::f,
                        tape_instruction: TapeInstruction::Right,
                        next_state: 2,
                    }],
                },
                State {
                    id: 2,
                    initial: false,
                    accepting: false,
                    transitions: vec![Transition {
                        match_char: AsciiChar::o,
                        tape_instruction: TapeInstruction::Right,
                        next_state: 3,
                    }],
                },
                State {
                    id: 3,
                    initial: false,
                    accepting: false,
                    transitions: vec![Transition {
                        match_char: AsciiChar::o,
                        tape_instruction: TapeInstruction::Right,
                        next_state: 4,
                    }],
                },
                State {
                    id: 4,
                    initial: false,
                    accepting: false,
                    transitions: vec![Transition {
                        match_char: AsciiChar::Null,
                        tape_instruction: TapeInstruction::Right,
                        next_state: 5,
                    }],
                },
                State {
                    id: 5,
                    initial: false,
                    accepting: true,
                    transitions: vec![],
                },
            ],
        })
        .unwrap();

        assert_tm(&tm, "foo", true)?;
        assert_tm(&tm, "food", false)?;
        Ok(())
    }
}
