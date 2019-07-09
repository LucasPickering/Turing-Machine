use crate::{
    ast::{Char, Program, State, StateId, Transition, ALPHABET_SIZE},
    error::{CompilerError, CompilerErrors},
};
use failure::Error;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::iter;
use std::ops::Deref;

/// A wrapper to indicate that the contents have been validated. This can only
/// be created via `Validate::validate_into`, to prevent tomfoolery.
#[derive(Debug)]
pub struct Valid<T: Debug + Sized>(T);

impl<T: Debug + Sized> Deref for Valid<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Defines validation behavior for a type. Some types require contextual data
/// for validation, such as a list of valid IDs. This trait defines a type
/// `Context` for that purpose.
pub trait Validate: Debug + Sized {
    type Context;

    /// Validates this object, using the given context. Any errors that are
    /// identified will be returned.
    fn validate(&self, context: &Self::Context) -> Vec<CompilerError>;

    /// Validates this object, and if it's valid, moves it into a `Valid`
    /// wrapper to indicate that.
    fn validate_into(
        self,
        context: &Self::Context,
    ) -> Result<Valid<Self>, Error> {
        let errors = self.validate(context);
        if errors.is_empty() {
            Ok(Valid(self))
        } else {
            Err(CompilerErrors::new(errors).into())
        }
    }
}

impl Validate for Program {
    type Context = ();

    fn validate(&self, _context: &Self::Context) -> Vec<CompilerError> {
        // Collect initial data that we'll need for validation
        let mut state_ids: HashMap<StateId, usize> = HashMap::new();
        for state in &self.states {
            // Count the number of occurrences of each state ID
            let state_id = state.id;
            state_ids
                .insert(state_id, state_ids.get(&state_id).unwrap_or(&0) + 1);
        }
        // Context for state validation
        let state_validation_ctx: (HashSet<StateId>,) =
            (state_ids.keys().copied().collect(),);

        // Most of the error checking is in this block
        let mut errors: Vec<CompilerError> = iter::empty()
            // Check for duplicate states
            .chain(
                state_ids
                    .iter()
                    .filter(|(_, count)| **count > 1)
                    .map(|(id, _)| CompilerError::DuplicateStateId(*id)),
            )
            // Validate each individual state (this also validates transitions)
            .chain(
                self.states
                    .iter()
                    .map(|state| state.validate(&state_validation_ctx))
                    .flatten(),
            )
            .collect();

        // Check that exactly one initial state is defined
        let initial_states: Vec<StateId> = self
            .states
            .iter()
            .filter(|state| state.initial)
            .map(|state| state.id)
            .collect();
        if initial_states.is_empty() {
            errors.push(CompilerError::NoInitialState);
        } else if initial_states.len() > 1 {
            errors.push(CompilerError::MultipleInitialStates(initial_states));
        }

        errors
    }
}

impl Validate for State {
    type Context = (HashSet<StateId>,);

    fn validate(&self, context: &Self::Context) -> Vec<CompilerError> {
        let mut errors = Vec::new();

        // Validate this ID
        if self.id == 0 {
            errors.push(CompilerError::InvalidStateId(self.id));
        }

        // Validate each transition
        errors.extend(
            self.transitions
                .iter()
                .map(|transition| transition.validate(context).into_iter())
                .flatten(),
        );
        errors
    }
}

impl Validate for Transition {
    type Context = (HashSet<StateId>,);

    fn validate(&self, context: &Self::Context) -> Vec<CompilerError> {
        let mut errors = Vec::new();
        // Validate the match char
        let match_char = self.match_char;
        if match_char == 0 || match_char >= (ALPHABET_SIZE as Char) {
            errors.push(CompilerError::IllegalCharacter(match_char));
        }

        // Validate the next state ID
        if !context.0.contains(&self.next_state) {
            errors.push(CompilerError::UndefinedState(self.next_state));
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::TapeInstruction;
    use crate::utils::assert_error;

    #[test]
    fn test_invalid_state_id_error() {
        let result = Program {
            states: vec![State {
                id: 0, // Invalid
                initial: true,
                accepting: true,
                transitions: vec![],
            }],
        }
        .validate_into(&());
        assert_error("Invalid state ID: 0", result);
    }

    #[test]
    fn test_duplicate_state_id_error() {
        let result = Program {
            states: vec![
                State {
                    id: 1,
                    initial: true,
                    accepting: true,
                    transitions: vec![],
                },
                State {
                    id: 1,
                    initial: false,
                    accepting: false,
                    transitions: vec![],
                },
            ],
        }
        .validate_into(&());
        assert_error("State ID defined multiple times: 1", result);
    }

    #[test]
    fn test_no_initial_state_error() {
        let result = Program {
            states: vec![State {
                id: 1,
                initial: false,
                accepting: true,
                transitions: vec![],
            }],
        }
        .validate_into(&());
        assert_error("No state marked as initial", result);
    }

    #[test]
    fn test_multiple_initial_states_error() {
        let result = Program {
            states: vec![
                State {
                    id: 1,
                    initial: true,
                    accepting: true,
                    transitions: vec![],
                },
                State {
                    id: 2,
                    initial: true,
                    accepting: true,
                    transitions: vec![],
                },
            ],
        }
        .validate_into(&());
        assert_error("Multiple states marked as initial: [1, 2]", result);
    }

    #[test]
    fn test_undefined_state_error() {
        let result = Program {
            states: vec![State {
                id: 1,
                initial: false,
                accepting: true,
                transitions: vec![Transition {
                    match_char: 32,
                    tape_instruction: TapeInstruction::Left,
                    next_state: 2, // Invalid
                }],
            }],
        }
        .validate_into(&());
        assert_error("Undefined state: 2", result);
    }

    #[test]
    fn test_char_zero_error() {
        let result = Program {
            states: vec![State {
                id: 1,
                initial: false,
                accepting: true,
                transitions: vec![Transition {
                    match_char: 0, // Invalid
                    tape_instruction: TapeInstruction::Left,
                    next_state: 1,
                }],
            }],
        }
        .validate_into(&());
        assert_error("Illegal character: \x00", result);
    }

    #[test]
    fn test_char_too_large_error() {
        let result = Program {
            states: vec![State {
                id: 1,
                initial: false,
                accepting: true,
                transitions: vec![Transition {
                    match_char: 0x80, // 128 - Invalid
                    tape_instruction: TapeInstruction::Left,
                    next_state: 1,
                }],
            }],
        }
        .validate_into(&());
        assert_error("Illegal character: \u{80}", result);
    }
}
