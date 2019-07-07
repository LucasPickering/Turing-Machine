use crate::{
    ast::{Char, Program, State, StateId, Transition, ALPHABET_SIZE},
    error::{CompilerError, CompilerResult},
};
use std::collections::{HashMap, HashSet};
use std::iter;

pub struct Valid<T: Sized>(pub T);

/// Defines validation behavior for a type. Some types require contextual data
/// for validation, such as a list of valid IDs. This trait defines a type
/// `Context` for that purpose.
pub trait Validate: Sized {
    type Context;

    /// Validates this object, using the given context. Any errors that are
    /// identified will be returned.
    fn validate(&self, context: &Self::Context) -> Vec<CompilerError>;

    /// Validates this object, and if it's valid, moves it into a `Valid`
    /// wrapper to indicate that.
    fn validate_into(
        self,
        context: &Self::Context,
    ) -> CompilerResult<Valid<Self>> {
        let errors = self.validate(context);
        if errors.is_empty() {
            Ok(Valid(self))
        } else {
            Err(errors)
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
            errors
                .push(CompilerError::InvalidCharacter(char::from(match_char)));
        }

        // Validate the next state ID
        if !context.0.contains(&self.next_state) {
            errors.push(CompilerError::StateDoesNotExist(self.next_state));
        }
        errors
    }
}
