use serde::{Deserialize, Serialize};

/// Number of bits used to represent one character in our alphabet.
/// Restricted to ASCII to maximize stack length when it gets encoded to a
/// single int.
pub const CHAR_SIZE_BITS: usize = 7;

/// The number of characters that our machine can recognize.
pub const ALPHABET_SIZE: u8 = 1 << CHAR_SIZE_BITS; // 1 << n == 2^n

pub type StateId = usize;

/// The different types of instructions that the TM can execute during a
/// transition.
///
/// This is not the most common way of defining a TM (usually you write AND
/// move in each transition), but this is how KG taught us, and who am I to
/// question him.
#[derive(Debug, Serialize, Deserialize)]
pub enum TapeInstruction {
    Left,
    Right,
    Write(char),
}

/// One transition, defined by a (state, char) pair. This consists of a tape
/// instruction, and a destination state.
#[derive(Debug, Serialize, Deserialize)]
pub struct Transition {
    /// The character on the tape that triggers this transition
    pub match_char: char,
    /// The instruction to execute on the tape (L/R/W)
    pub tape_instruction: TapeInstruction,
    /// The state to transition to next
    pub next_state: StateId,
}

/// One state in the machine.
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    /// Unique numerical ID for this state (starts at 0)
    pub id: StateId,
    /// Is this the initial state? Should be true for exactly one state in
    /// a machine.
    pub initial: bool,
    /// Is this an accepting state?
    pub accepting: bool,
    /// All transitions that can be made from this state
    pub transitions: Vec<Transition>,
}

/// An entire Turing machine program. The root of the AST.
#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub states: Vec<State>,
}
