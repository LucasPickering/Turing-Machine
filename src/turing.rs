use crate::stack::SmInstruction::{self, *};
use crate::stack::StackMachine;
use std::io;

/// The general strategy use here,  and almost all of the Rocketlang code, was
/// created by Dr. Kevin Gold. The strategy is to simulate a two-stack PDA by
/// essentially encoding one stack as a single number and passing that around
/// between the two variables and the stack. Generally speaking, the stack holds
/// the right half of the tape, including the piece under the head, and one of
/// the variables holds the left half of the tape, encoded as a single int.
/// Obviously, all the data has to get passed around a lot to be able to make
/// room for computations.

// Will be truncated to 7 bits (ASCII) to maximize stack length when encoded to an int
type Char = u8;

/// This is not the most common way of defining a TM (usually you write AND
/// move in each transition), but this is how KG taught us, and who am I to
/// question him.
enum TapeInstruction {
    Left,
    Right,
    Write(Char),
}

struct State<'a> {
    /// Unique numerical ID for this state
    id: u32,
    /// All transitions that can be made from this state
    transitions: Vec<Transition<'a>>,
}

struct Transition<'a> {
    /// The character on the tape that triggers this transition
    input_char: Char,
    /// The instruction to execute on the tape (L/R/W)
    tape_instruction: TapeInstruction,
    /// The state to transition to next
    next_state: &'a State<'a>,
}

/// A Turing machine built entirely on Rocketlang's stack machine. This proves
/// that Rocketlang is Turing-complete.
///
/// This has the external API of a TM, but internally only uses the two-variable
/// stack machine from Rocketlang.
///

pub struct TuringMachine {
    instructions: Vec<SmInstruction>,
}

impl TuringMachine {
    pub fn new(states: &[State]) -> Self {
        Self {
            instructions: Self::compile(states),
        }
    }

    pub fn run(&self, input: String) {
        let mut machine = StackMachine::new(input.as_bytes(), io::stdout());
        machine.run(&self.instructions);
    }

    fn compile(states: &[State]) -> Vec<SmInstruction> {
        vec![
            // -------
            // PRELUDE
            // -------
            // Read the input string onto the tape. For convenience, assume the
            // input is reversed and terminated with a 0, e.g. "foo" is
            // actually "oof\x00".
            ReadToActive,
            While(vec![PushActive, ReadToActive]),
            PushZero, // Unclear on why this is here - why do we need two blanks?
            // -----
            // Now the stack will hold the portion of the tape at and right of
            // the head. var_active will hold the state number and var_inactive
            // will have the tape left of the head (encoded as a number).
            IncrActive, // Set initial state to 1
            // TODO Allow specifying initial state
            // ---------
            // MAIN LOOP
            // ---------
            Self::compile_main_loop(states),
            // --------
            // POSTLUDE
            // --------
        ]
    }

    fn compile_main_loop(states: &[State]) -> SmInstruction {
        // TM state at the start of each iteration:
        // var_active: State ID #
        // var_inactive: Encoded left end of tape (excl. head)
        // Stack:
        // --

        let instrs = Vec::new();

        // Prelude
        instrs.append(&mut vec![
            // Store var_active (left tape) on the stack, then reset it to 0
            Swap,
            PushActive,
            PushZero,
            PopToActive,
            Swap,
        ]);

        // Generate code for each state and add it to the loop. Exactly one
        // state will be executed on each iteration, or if none match, then
        // we'll terminate.
        for state in states {
            instrs.append(&mut Self::compile_state(state))
        }

        While(instrs)
    }

    fn compile_state(state: &State) -> Vec<SmInstruction> {
        // var_active holds the state counter. It starts at n (current state
        // #), and counts down to 0. Once it hits 0, it will be at the nth
        // state block, so the If condition will match and it will execute.

        // var_inactive is 0 and free to use
        // Stack is:
        // -- Left tape (encoded)
        // -- Character under tape head (head char)
        // -- ...Right tape

        let state_body = Vec::new();

        // Setup logic for switching on the head char
        state_body.append(&mut vec![
            PopToActive, // Pop the left tape to var_active
            Swap, // Move left tape to var_inactive, var_active is now empty
            PopToActive, // var_active has the head char
            // Stack is now JUST the right tape
            Swap,       // var_active has the left tape again
            PushActive, // Left tape is back on stack (still encoded)
        ]);

        // TM state now:
        // var_active: Garbage
        // var_inactive: Head char
        // Stack:
        // -- Left tape (encoded)
        // -- ...Right tape

        // Now we're going to check for a transition on each character.
        // For each char that we have a transition for, we reset var_active to
        // 0, then incr up to the char value and use an If to check against
        // the head char.
        // TODO: Optimize this by sorting the chars and not resetting each time

        for transition in state.transitions {
            state_body.append(&mut vec![
                // Reset var_active to 0
                PushZero,
                PopToActive,
            ]);

            // Set var_active = c
            for _ in 0..transition.input_char {
                state_body.push(IncrActive);
            }

            // Add the write/move/next state code for this transition
            state_body.push(If(vec![]));
        }

        vec![DecrActive, If(state_body)]
    }
}
