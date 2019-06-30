use crate::stack::SmAction;
use crate::stack::StackMachine;
use std::collections::HashMap as Map;
use std::io;

type State = String;
type Char = u8; // Will be truncated to 7 bits - ASCII only

pub enum Direction {
    Left,
    Right,
}

/// The conditions used to look up which state to run.
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
        vec![
            // Read the input string onto the tape. For convenience, assume the
            // input is reversed and terminated with a 0, e.g. "foo" is
            // actually "oof0".
            SmAction::ReadToActive,
            SmAction::While(vec![SmAction::PushActive, SmAction::ReadToActive]),
            SmAction::Push0,
            // Now the stack will hold the portion of the tape at and right of
            // the head. One counter will have the tape left of the head
            // (encoded as a number) and the other counter will have the state
            SmAction::IncrActive,
            // The main loop
            SmAction::While(vec![]),
        ]
    }
}
