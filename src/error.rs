use failure::Fail;

#[derive(Debug, Fail)]
pub enum CompilerError {
    #[fail(display = "Invalid character: {}", 0)]
    InvalidCharacter(char),
}

pub type CompilerResult<T> = Result<T, Vec<CompilerError>>;
