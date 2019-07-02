use crate::stack::SmInstruction::{self, *};
use crate::stack::StackMachine;
use std::io;
use std::iter;

/// The general strategy use here,  and almost all of the Rocketlang code, was
/// created by Dr. Kevin Gold. The strategy is to simulate a two-stack PDA by
/// essentially encoding one stack as a single number and passing that around
/// between the two variables and the stack. Generally speaking, the stack holds
/// the right half of the tape, including the piece under the head, and one of
/// the variables holds the left half of the tape, encoded as a single int.
/// Obviously, all the data has to get passed around a lot to be able to make
/// room for computations.

// Will be truncated to 7 bits (ASCII) to maximize stack length when encoded to an int
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
    /// Unique numerical ID for this state
    id: u32,
    /// All transitions that can be made from this state
    transitions: Vec<Transition<'a>>,
}

pub struct Transition<'a> {
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

    /// Generates a While instruction to be the main loop of the program.
    /// Includes logic for each state and transition.
    fn compile_main_loop(states: &[State]) -> SmInstruction {
        // TM state at the start of each iteration:
        // var_active: State ID #
        // var_inactive: Encoded left end of tape (excl. head)
        // Stack:
        // --

        While(
            // Prelude
            vec![
                // Store var_active (left tape) on the stack, then reset it to 0
                Swap,
                PushActive,
                PushZero,
                PopToActive,
                Swap,
            ]
            .into_iter()
            // Generate code for each state and add it to the loop. Exactly one
            // state will be executed on each iteration, or if none match, then
            // we'll terminate.
            .chain(states.iter().map(Self::compile_state).flatten())
            .collect(),
        )
    }

    /// Compiles logic for a single state, including the Decr needed to check
    /// before the If, and the If itself with all internal logic.
    fn compile_state(state: &State) -> Vec<SmInstruction> {
        // var_active holds the state counter. It starts at n (current state
        // #), and counts down to 0. Once it hits 0, it will be at the nth
        // state block, so the If condition will match and it will execute.

        // var_inactive is 0 and free to use
        // Stack is:
        // -- Left tape (encoded)
        // -- Character under tape head (head char)
        // -- ...Right tape

        // Setup logic for switching on the head char
        vec![
            DecrActive, // Step down to the next state ID that we're checking
            If(vec![
                PopToActive, // Pop the left tape to var_active
                Swap, // Move left tape to var_inactive, var_active is now empty
                PopToActive, // var_active has the head char
                // Stack is now JUST the right tape
                Swap,       // var_active has the left tape again
                PushActive, // Left tape is back on stack (still encoded)
            ]
            .into_iter()
            // var_active: Trash
            // var_inactive: Head char
            // Stack:
            // -- Left tape (encoded)
            // -- ...Right tape
            // Now we're going to check for a transition on each character.
            // For each char that we have a transition for, we reset var_active to
            // 0, then incr up to the char value and use an If to check against
            // the head char.
            // TODO: Optimize this by sorting the chars and not resetting each time
            // NOTE: The logic here for iterating over the characters is slightly
            // different from KG's version (I thought this was simpler).
            .chain(
                state
                    .transitions
                    .iter()
                    .map(Self::compile_transition_check)
                    .flatten(),
            )
            .collect()),
        ]
    }

    /// Generates code to check the next character transition, and execute that
    /// transition if it matches.
    fn compile_transition_check(transition: &Transition) -> Vec<SmInstruction> {
        vec![
            // Reset var_active to 0
            PushZero,
            PopToActive,
        ]
        .into_iter()
        // Set var_active = transition_char
        .chain(iter::repeat(IncrActive).take(transition.input_char as usize))
        // var_active: Transition char (char to match head against)
        // var_inactive: Head char
        // Stack:
        // -- Left tape (encoded)
        // -- ...Right tape
        // Add the write/move/next state code for this transition.
        // This will execute only if the transition char matches the head.
        // Once the If matches, we know var_active = var_inactive, so we
        // can trash one.
        .chain(iter::once(If(Self::compile_transition(transition))))
        .collect()
    }

    /// Compiles a singular state transition into a series of instructions.
    /// This is non-conditional: if these instructions are executing, we know
    /// the transition is valid. This will apply the relevant tape instruction,
    /// then set the next state.
    fn compile_transition(transition: &Transition) -> Vec<SmInstruction> {
        // var_active: Trash
        // var_inactive: Head char
        // Stack:
        // -- Left tape (encoded)
        // -- ...Right tape

        vec![
            PopToActive, // Pop left tape
            Swap,
            PushActive, // Push head char
        ]
        .into_iter()
        // var_active: Trash
        // var_inactive: Left tape (encoded)
        // Stack:
        // -- Head char
        // -- ...Right tape
        .chain(Self::compile_tape_instruction(&transition.tape_instruction))
        // TODO: Go to next state
        .collect()
    }

    /// Compiles a single tape instruction (L/R/W) into a series of SM
    /// instructions. This returns a dynamic type so that the different match
    /// arms can return different iterators.
    fn compile_tape_instruction(
        tape_instruction: &TapeInstruction,
    ) -> Vec<SmInstruction> {
        match tape_instruction {
            TapeInstruction::Left => vec![],  // TODO
            TapeInstruction::Right => vec![], // TODO

            TapeInstruction::Write(c) => vec![
                // Remove the head char and replace it with a new one
                PopToActive,
                PushZero,
                PopToActive,
            ]
            .into_iter()
            .chain(iter::repeat(IncrActive).take(*c as usize))
            .chain(vec![PushActive])
            .collect(),
            // var_active: Trash
            // var_inactive: Left tape (encoded)
            // Stack:
            // -- NEW head char
            // -- ...Right tape
        }
    }
}
