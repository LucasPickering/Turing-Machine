use crate::ast::StateId;
use failure::{ Fail};
use itertools::Itertools;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Fail)]
pub enum CompilerError {
    #[fail(display = "Invalid state ID: {}. Must be >0.", 0)]
    InvalidStateId(StateId),
    #[fail(display = "State ID defined multiple times: {}", 0)]
    DuplicateStateId(StateId),
    #[fail(display = "No state marked as initial")]
    NoInitialState,
    #[fail(display = "Multiple states marked as initial: {:?}", 0)]
    MultipleInitialStates(Vec<StateId>),
    #[fail(display = "Undefined state: {}", 0)]
    UndefinedState(StateId),
    #[fail(display = "Invalid character: {}", 0)]
    InvalidCharacter(char),
}

// Container for holding multiple compiler errors. This is the most common way
// to report errors.
#[derive(Debug, Fail)]
pub struct CompilerErrors(Vec<CompilerError>);

impl CompilerErrors {
    pub fn new(errors: Vec<CompilerError>) -> Self {
        CompilerErrors(errors)
    }
}

impl Display for CompilerErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().join("\n"))
    }
}

impl Deref for CompilerErrors {
    type Target = Vec<CompilerError>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
