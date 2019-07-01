use crate::stack::SmAction::{self, *};
use crate::stack::StackMachine;
use std::collections::HashMap as Map;
use std::io;

type State = u32;
type Char = u8; // Will be truncated to 7 bits - ASCII only

pub enum Direction {
    Left,
    Right,
}

/// The conditions used to look up which state to run next
/// (current state and tape char)
type InstructionLookup = (State, Char);

/// Instruction executed - character to write, then direction to move.
type Instruction = (Char, Direction);

/// A Turing machine built entirely on Rocketlang's stack machine. This proves
/// that Rocketlang is Turing-complete.
///
/// This has the external API of a TM, but internally only uses the two-variable
/// stack machine from Rocketlang.
pub struct TuringMachine {
    actions: Vec<SmAction>,
}

impl TuringMachine {
    pub fn new(instruction_table: Map<InstructionLookup, Instruction>) -> Self {
        Self {
            actions: Self::compile(instruction_table),
        }
    }

    pub fn run(&self, input: String) {
        let mut machine = StackMachine::new(input.as_bytes(), io::stdout());
        machine.run(&self.actions);
    }

    fn compile(
        instruction_table: Map<InstructionLookup, Instruction>,
    ) -> Vec<SmAction> {
        // Setup code
        let prelude = vec![
            // Read the input string onto the tape. For convenience, assume the
            // input is reversed and terminated with a 0, e.g. "foo" is
            // actually "oof\x00".
            ReadToActive,
            While(vec![PushActive, ReadToActive]),
            PushZero,
            // -----
            // Now the stack will hold the portion of the tape at and right of
            // the head. var1 will hold the state number and var2 will have the
            // tape left of the head (encoded as a number).
            // Set the current state to 1, just so the loop will run
            IncrActive,
            // -----
            // The main loop, runs as long as the state isn't 0
            While(vec![
                // Store var2 (left tape) on the stack, then reset it to 0
                Swap,
                PushActive,
                PushZero,
                PopToActive,
                Swap,
                DecrActive,
            ]),
        ];

        // Teardown code
        let postlude = vec![];

        // Put it all together
        let mut rv = Vec::new();
        rv.extend_from_slice(&prelude);
        rv.extend_from_slice(&postlude);
        rv
    }
}
