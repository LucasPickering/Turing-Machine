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
    #[fail(display = "Undefined state: {}", 0)]
    UndefinedState(StateId),
    #[fail(display = "Invalid character: {}", 0)]
    InvalidCharacter(char),
}

pub type CompilerResult<T> = Result<T, Vec<CompilerError>>;
