use crate::{
    stack::SmInstruction::{self, *},
    turing::{Char, State, TapeInstruction, Transition, ALPHABET_SIZE},
};
use itertools::Itertools;
use std::{collections::HashMap, iter};

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

/// Defines compilation steps for a single type.
pub trait Compile {
    /// Generates a sequence of instructions that execute the steps necessary
    /// to process this data type.
    fn compile(&self) -> Vec<SmInstruction>;
}

impl<'a> Compile for [State<'a>] {
    /// Compiles the given Turing Machine (represented by a series of states)
    /// into a series of stack machine instructions.
    fn compile(&self) -> Vec<SmInstruction> {
        vec![
            // -------
            // PRELUDE
            // -------
            ToggleErrors, // Disable errors
            // Read the input string onto the tape. For convenience, assume the
            // input is reversed and terminated with a 0, e.g. "foo" is
            // actually "oof\x00". The \x00 never ends up on the stack though.
            // TODO Fix this by abusing ReadToActive without errors
            ReadToActive,
            While(vec![PushActive, ReadToActive]),
            PushZero, // Set initial left tape to 0
            //
            // Now the stack will hold the portion of the tape at and right of
            // the head. var_a will hold the state number and var_i
            // will have the tape left of the head (encoded as a number).
            IncrActive, // Set initial state to 1
            //
            // TODO Allow specifying initial state
            // ---------
            // MAIN LOOP
            // ---------
            // var_a: Initial state ID
            // var_i: 0
            // - 0 (Initial left tape)
            // - Head char
            // - ...Right tape
            While(
                // TM state at the start of each iteration:
                // var_a: Current (i.e. desired) state #
                // var_i: 0
                // - Left tape (encoded)
                // - Head char
                // - ...Right tape

                // Generate code for each state and add it to the loop. Exactly
                // one state will be executed on each iteration, or if none
                // match, then we'll halt. See State::compile for more on how
                // this works, and why we have to sort the states.
                self.iter()
                    .sorted_by_key(|state| state.id)
                    .map(State::compile)
                    .flatten()
                    // var_a: FREE
                    // var_i: 0
                    // - Next state #
                    // - Left tape (encoded)
                    // - Head char
                    // - ...Right tape
                    // Get the next state off the stack
                    .chain(vec![PopToActive])
                    .collect(),
            ),
            /*
             * --------
             * POSTLUDE
             * -------- */
        ]
    }
}

impl<'a> Compile for State<'a> {
    /// Compiles logic for a single state, including the If with all internal logic
    /// and the following Decr to step to the next state to check.
    ///
    /// If this state executes, both variables will be reset to 0. Because a
    /// Decr will occur before the next state If, no subsequent state Ifs will
    /// match.
    ///
    /// ## Input state
    /// var_a: State counter
    /// var_i: 0
    /// - Left tape (encoded)
    /// - Head char
    /// - ...Right tape
    ///
    /// ## Output state
    /// ### If this state executes:
    /// var_a: 0
    /// var_i: 0
    /// - Next state #
    /// - Left tape (encoded)
    /// - Head char
    /// - ...Right tape
    ///
    /// ### If it doesn't execute:
    /// var_a: State counter
    /// var_i: 0
    /// - Left tape (encoded)
    /// - Head char
    /// - ...Right tape
    fn compile(&self) -> Vec<SmInstruction> {
        // The state counter starts at n (desired state #), and counts down to 0.
        // It will hit 0 on the nth state check, and the If condition will match.
        // This means the states have to be sorted by ascending ID, so that State n
        // is the nth block.

        // Setup logic for switching on the head char
        vec![
            DecrActive, // Step "up" to the next state ID that we're checking
            If(
                // State body
                vec![
                    PopToActive, // Pop the left tape to var_a
                    Swap,        // Move left tape to var_i, var_a is now empty
                    PopToActive, // var_a has the head char
                    // Stack is now JUST the right tape
                    Swap,       // var_a has the left tape again
                    PushActive, // Left tape is back on stack (still encoded)
                    // Reset to 0
                    PushZero,
                    PopToActive,
                ]
                .into_iter()
                // Generate a big list of Ifs, one for each transition
                .chain(self.transitions.compile())
                .collect(),
            ),
        ]
    }
}

impl<'a> Compile for [Transition<'a>] {
    /// Compiles the given transitions into a set of If statements with some
    /// logic to count through them and match the correct one.
    ///
    /// ## Input state
    /// var_a: 0
    /// var_i: Head char
    /// - Left tape (encoded)
    /// - ...Right tape
    ///
    /// ## Output state
    /// var_a: 0
    /// var_i: 0
    /// - Next state #
    /// - Left tape (encoded)
    /// - Head char
    /// - ...Right tape
    fn compile(&self) -> Vec<SmInstruction> {
        // Now we're going to check for a transition on each character. Start at
        // 0 and count up until we hit the char we're looking for. Note that,
        // like states, we have to sort the characters so that we can count up
        // through them. Unlike states though, transition chars aren't
        // guaranteed to be contiguous so we have to fill the gaps with extra
        // incrs.
        // e.g. if we have transitions for c=0 and c=2, we need two incrs
        // between the two Ifs to properly match the second condition.
        //
        // NOTE: The logic here for iterating over the characters is slightly
        // different from KG's version (I thought this was simpler). He wanted
        // to decr from the head char, but then we're trashing it unnecessarily
        // and need to include extra Incrs to get it back.

        let keyed_by_char: HashMap<Char, &Transition> = self
            .iter()
            .map(|transition| (transition.match_char, transition))
            .collect();

        // For every char in the range we want to check, if there is a
        // transition for that char, add code for the transition. For EVERY
        // char, even ones without transitions, add an Incr so we can progress
        // to the next char.
        (0..ALPHABET_SIZE)
            .map(|c| {
                // If there is a transition for this char, compile it. If not,
                // just add an Incr and move on.
                let mut instrs = Vec::new();

                if let Some(transition) = keyed_by_char.get(&(c as u8)) {
                    instrs.push(If(transition.compile()));
                }
                instrs.push(IncrActive);
                instrs
            })
            .flatten()
            // Two possible states here. If a transition above executed:
            // var_a: FREE
            // var_i: -1
            // - Next state #
            // - Left tape (encoded)
            // - Head char
            // - ...Right tape
            //
            // If no transitions executed (because none of them matched):
            // var_a: ALPHABET_SIZE
            // var_i: Head char
            // - Left tape (encoded)
            // - ...Right tape
            .chain(vec![
                // Catch-all transition to handle the case where none of the
                // other transitions matched. We can use a loop to tell if var_i
                // is >=0. Loops check var_a with >0, so we need a Swap and Incr.
                Swap,
                IncrActive, // In case Head char == 0
                While(vec![
                    DecrActive,  // Undo the Incr now that we know we entered
                    Swap,        // var_a is free now
                    PopToActive, // Pop LT
                    Swap,        // var_a = HC, var_i = LT
                    PushActive,  // Push HC
                    Swap,        // var_a = LT, var_i = HC
                    PushActive,  // Push LT
                    PushZero,    // Set next state = 0 (will cause a HALT)
                    // Reset var_a=0 so we exit the loop, and var_i=0 because
                    // our output contract specifies that.
                    PushZero,
                    PopToActive,
                    Swap,
                    PushZero,
                    PopToActive,
                ]),
            ])
            .collect()
    }
}

impl<'a> Compile for Transition<'a> {
    /// Generates code to execute a transition, which includes one of a L/R/W,
    /// plus setting the next state.
    ///
    /// After this runs, var_a is reset to 0, and var_i is set to -1 (an invalid
    /// char value) to indicate that the transition executed. Only Incrs will
    /// run after this If, so from here on var_a > var_i, making it easy to
    /// tell if a transition executed at the end.
    ///
    /// ## Input state
    /// var_a: Transition char counter
    /// var_i: Head char
    /// - Left tape (encoded)
    /// - ...Right tape
    ///
    /// ## Output state
    /// var_a: 0
    /// var_i: -1
    /// - Next state #
    /// - Left tape (encoded) [MODIFIED]
    /// - Head char [MODIFIED]
    /// - ...Right tape [MODIFIED]
    fn compile(&self) -> Vec<SmInstruction> {
        // Add the write/move/next state code for this transition.
        // This will execute only if the transition char matches the head.
        // Once the If matches, we know var_a = var_i, so we can trash one.
        vec![
            PopToActive, // Pop left tape
            Swap,
            PushActive, // Push head char
        ]
        .into_iter()
        .chain(self.tape_instruction.compile())
        .chain(vec![
            // Push LT back on the stack
            Swap,
            PushActive,
            // Reset one variable
            PushZero,
            PopToActive,
            Swap,
            // Reset the other
            PushZero,
            PopToActive,
        ])
        // var_a: 0
        // var_i: 0
        // - Left tape (encoded)
        // - Head char
        // - ...Right tape
        // Set the next state and push it onto the stack.
        .chain(iter::repeat(IncrActive).take(self.next_state.id as usize))
        .chain(vec![PushActive, PushZero, PopToActive])
        .collect()
    }
}

impl Compile for TapeInstruction {
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
    fn compile(&self) -> Vec<SmInstruction> {
        match self {
            // Strategy here: Divide left tape by alphabet SIZE by repeated
            // subtracting SIZE until we get negative, then adding it back
            // once. This will give us a remainder, which is the rightmost
            // character on the left tape (i.e. our new head). Extract that,
            // then undo the division by adding SIZE back the same number
            // of times. Here's a math proof to make it seem more believable:

            // LT: Current Left Tape value
            // LT': Left Tape value after the shift
            // SIZE: Alphabet size, i.e. 2^n where n is the char size in bits
            // H: Head Char value after the shift
            // x: Number of times we subtract SIZE from LT to make LT<=0
            // R: Remainder after computing (LT / SIZE) (i.e. val of lowest n bits)

            // EQ1:
            // LT = (LT' * SIZE) + H         : By definition of the tape
            // -----
            // EQ2:
            // LT - (SIZE * x) = R - SIZE    : By the method we use to compute R
            // LT = R - SIZE + (SIZE * x)
            // LT = R + (-1 + x) * SIZE
            // LT = SIZE(x - 1) + R
            // -----
            // Therefore:
            // H = R
            // LT' = x - 1
            TapeInstruction::Left => vec![
                PushZero,
                PopToActive,
                Swap,
                While(
                    // State before each iteration:
                    // var_a: Left tape (partially divided)
                    // var_i: Decr counter (# of times we've subtracted SIZE from LT)
                    // - ...Right tape
                    iter::repeat(DecrActive)
                        .take(ALPHABET_SIZE)
                        .chain(vec![Swap, IncrActive, Swap])
                        .collect(),
                ),
                // This terminates when LT goes negative, so state is now:
                // var_a: LT remainder minus SIZE (i.e. with one extra subtraction)
                // var_i: Decr counter (# of times we subtracted SIZE from LT)
                // - ...Right tape
            ]
            .into_iter()
            // Add SIZE back to the remainder to get the new Head value
            .chain(iter::repeat(IncrActive).take(ALPHABET_SIZE))
            // This won't terminate until LT goes negative, so state is:
            // var_a: LT remainder (i.e. NEW head char)
            // var_i: Decr counter (# of times we subtracted SIZE from LT)
            // - ...Right tape
            .chain(vec![
                // Push new head char, then reset that counter to 0
                PushActive, Swap,
                // Now var_a holds the number of times we subtracted SIZE. That
                // will be one greater than the new value of LT, so just decr.
                // Swap back to put LT back in var_i
                DecrActive, Swap,
            ])
            .collect(),

            TapeInstruction::Right => iter::repeat(vec![
                // Similar to left shift, we have to do some tedious math to add a
                // char to the left tape.
                // First, we need to free up the bottom n bits in the left tape,
                // where n is the number of bits in a char. Just do LT << n.
                // Oh wait... we don't have bit ops. Let's do LT * 2^n. Shit.
                // Don't have that either. Guess we have to add LT to itself
                // (2^n)-1 times. Seems tractable enough.

                // Load LT into var_a (and in var_i)
                Swap,
                PushActive,
                Swap,
                PopToActive,
                // Add LT to var_i
                While(vec![DecrActive, Swap, IncrActive, Swap]),
            ])
            .take(ALPHABET_SIZE - 1)
            .flatten()
            // Now put the head char back in var_a, then add its value to the
            // lowest n bits of the left tape
            .chain(vec![Swap, While(vec![DecrActive, Swap, IncrActive, Swap])])
            .collect(),

            TapeInstruction::Write(c) => {
                // Pop the head char to var_a, then reset to 0
                vec![PopToActive, PushZero, PopToActive]
                    .into_iter()
                    // Incr up to the new char value, then push it
                    .chain(iter::repeat(IncrActive).take(*c as usize))
                    .chain(vec![PushActive])
                    .collect()
            }
        }
    }
}
