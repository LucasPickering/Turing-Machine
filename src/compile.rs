use crate::{
    stack::SmInstruction::{self, *},
    turing::{State, TapeInstruction, Transition},
};
use std::iter;

/// Stack machine codegen logic. There are many places where we want to
/// document the expected machine state, so we'll standardize a way of
/// specifying that:
///
/// ```
/// var_a: A
/// var_i: I
/// - X
/// - Y
/// - ...L
/// ```
///
/// - `A` is the active variable
/// - `I` is the inactive variable
/// - `X` is a single element, on top of the stack
/// - `Y` is a single element, below `X` on the stack
/// - `L` is a list of elements that fills the rest of the stack
///
/// Also, the term "head char" refers to the character on the tape that's
/// under the head of the machine.

/// Compiles the given Turing Machine (represented by a series of states) into
/// a series of stack machine instructions.
pub fn compile(states: &[State]) -> Vec<SmInstruction> {
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
        //
        // Now the stack will hold the portion of the tape at and right of
        // the head. var_active will hold the state number and var_inactive
        // will have the tape left of the head (encoded as a number).
        IncrActive, // Set initial state to 1
        //
        // TODO Allow specifying initial state
        // ---------
        // MAIN LOOP
        // ---------
        compile_main_loop(states),
        //
        // --------
        // POSTLUDE
        // --------
    ]
}

/// Generates a While instruction to be the main loop of the program.
/// Includes logic for each state and transition.
///
/// ## Input state
/// var_a: Initial state ID
/// var_i: FREE
/// - Head char
/// - ...Right tape
///
/// ## Output state
/// TODO
fn compile_main_loop(states: &[State]) -> SmInstruction {
    While(
        // TM state at the start of each iteration:
        // var_a: Current state ID
        // var_i: Left tape (encoded)
        // - Head char
        // - ...Right tape

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
        .chain(states.iter().map(compile_state).flatten())
        .collect(),
    )
}

/// Compiles logic for a single state, including the Decr needed to check
/// before the If, and the If itself with all internal logic.
///
/// ## Input state
/// var_a: State counter
/// var_i: FREE
/// - Left tape (encoded)
/// - Head char
/// - ...Right tape
///
/// ## Output state
/// TODO
fn compile_state(state: &State) -> Vec<SmInstruction> {
    // The state counter starts at n (current state #), and counts down to 0.
    // Once it hits 0, it will be at the nth state block, so the If condition
    // will match and it will execute.

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
        // var_a: FREE
        // var_i: Head char
        // - Left tape (encoded)
        // - ...Right tape
        //
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
                .map(compile_transition_check)
                .flatten(),
        )
        .collect()),
    ]
}

/// Generates code to check the next character transition, and execute that
/// transition if it matches.
///
/// ## Input state
/// var_a: FREE
/// var_i: Head char
/// - Left tape (encoded)
/// - ...Right tape
///
/// ## Output state
/// Varies depending on whether or not this transition matches.
///
/// ### If it matches
/// var_a: FREE
/// var_i: Left tape (encoded - MODIFIED)
/// - Head char (MODIFIED)
/// - ...Right tape (MODIFIED)
///
/// If this transition matches, no
///
/// ### If it doesn't
/// var_a: FREE
/// var_i: Head char
/// - Left tape (encoded)
/// - ...Right tape (encoded)
fn compile_transition_check(transition: &Transition) -> Vec<SmInstruction> {
    vec![
        // Reset var_active to 0
        PushZero,
        PopToActive,
    ]
    .into_iter()
    // Set var_active = transition_char
    .chain(iter::repeat(IncrActive).take(transition.input_char as usize))
    // var_a: Transition char (char to match head against)
    // var_i: Head char
    // - Left tape (encoded)
    // - ...Right tape
    //
    // Add the write/move/next state code for this transition.
    // This will execute only if the transition char matches the head.
    // Once the If matches, we know var_active = var_inactive, so we
    // can trash one.
    .chain(iter::once(If(compile_transition(transition))))
    .collect()
}

/// Compiles a singular state transition into a series of instructions.
/// This is non-conditional: if these instructions are executing, we know
/// the transition is valid. This will apply the relevant tape instruction,
/// then set the next state.
///
/// ## Input state
/// var_a: FREE
/// var_i: Head char
/// - Left tape (encoded)
/// - ...Right tape
///
/// ## Output state
/// var_a: FREE
/// var_i: Left tape (encoded - MODIFIED)
/// - Head char (MODIFIED)
/// - ...Right tape (MODIFIED)
fn compile_transition(transition: &Transition) -> Vec<SmInstruction> {
    vec![
        PopToActive, // Pop left tape
        Swap,
        PushActive, // Push head char
    ]
    .into_iter()
    .chain(compile_tape_instruction(&transition.tape_instruction))
    // TODO: Go to next state
    .collect()
}

/// Compiles a single tape instruction (L/R/W) into a series of SM
/// instructions. This returns a dynamic type so that the different match
/// arms can return different iterators.
///
/// After these instructions are executed, the tape will be modified because
/// of a shift or write.
///
/// ## Input state
/// var_a: FREE
/// var_i: Left tape (encoded)
/// - Head char
/// - ...Right tape
///
/// ## Output state
/// var_a: FREE
/// var_i: Left tape (encoded - MODIFIED)
/// - Head char (MODIFIED)
/// - ...Right tape (MODIFIED)
fn compile_tape_instruction(
    tape_instruction: &TapeInstruction,
) -> Vec<SmInstruction> {
    match tape_instruction {
        TapeInstruction::Left => vec![],  // TODO
        TapeInstruction::Right => vec![], // TODO

        TapeInstruction::Write(c) => vec![
            // Remove the head char and reset the counter to 0
            PopToActive,
            PushZero,
            PopToActive,
        ]
        .into_iter()
        // Incr up to the new char value, then push it
        .chain(iter::repeat(IncrActive).take(*c as usize))
        .chain(iter::once(PushActive))
        .collect(),
    }
}
