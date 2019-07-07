use crate::ast::StateId;
use failure::Fail;

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
    #[fail(display = "State does not exist: {}", 0)]
    StateDoesNotExist(StateId),
    #[fail(display = "Invalid character: {}", 0)]
    InvalidCharacter(char),
}

pub type CompilerResult<T> = Result<T, Vec<CompilerError>>;
