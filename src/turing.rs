use crate::{
    compile::{Compile, ContextCompile},
    stack::{SmInstruction, StackMachine},
};
use std::io;

/// Number of bits used to represent one character in our alphabet.
/// Restricted to ASCII to maximize stack length when it gets encoded to a
/// single int.
pub const CHAR_SIZE_BITS: usize = 7;

/// The number of characters that our machine can recognize.
pub const ALPHABET_SIZE: usize = 1 << CHAR_SIZE_BITS; // 1 << n == 2^n

/// Will be truncated to 7 bits to fit in the alphabet.
pub type Char = u8;

/// This is not the most common way of defining a TM (usually you write AND
/// move in each transition), but this is how KG taught us, and who am I to
/// question him.
pub enum TapeInstruction {
    Left,
    Right,
    Write(Char),
}

pub struct State<'a> {
    /// Unique numerical ID for this state (starts at 0)
    pub id: u32,
    /// All transitions that can be made from this state
    pub transitions: Vec<Transition<'a>>,
}

pub struct Transition<'a> {
    /// The character on the tape that triggers this transition
    pub match_char: Char,
    /// The instruction to execute on the tape (L/R/W)
    pub tape_instruction: TapeInstruction,
    /// The state to transition to next
    pub next_state: &'a State<'a>,
}

/// A Turing machine built entirely on Rocketlang's stack machine. This proves
/// that Rocketlang is Turing-complete.
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
    pub fn new(states: &[State], initial_state: u32) -> Self {
        Self {
            instructions: states.compile((initial_state,)),
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

    #[test]
    fn test_noop() {
        let state = State {
            id: 0,
            transitions: vec![],
        };
        let tm = TuringMachine::new(&[state], 0);
        assert!(!tm.instructions.is_empty());
        tm.run("\x00".to_owned());
    }
}
