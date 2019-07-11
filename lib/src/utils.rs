use crate::{ast::ALPHABET_SIZE, error::CompilerError};

pub fn validate_char(c: char) -> Result<(), CompilerError> {
    // Cast both to usize to make sure we don't truncate the character
    if c == '\x00' || c as usize >= ALPHABET_SIZE as usize {
        Err(CompilerError::IllegalCharacter(c))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use failure::Error;
    use std::fmt::Debug;

    /// Utility function for testing
    pub fn assert_error<T: Debug>(msg: &str, result: Result<T, Error>) {
        assert!(result.is_err());
        match result {
            Ok(_) => panic!("Expected Err but received Ok!"),
            Err(error) => {
                let err_str = error.to_string();
                if !err_str.contains(msg) {
                    panic!(
                        "Expected error {:?} to contain substring \"{}\"",
                        err_str, msg
                    );
                }
            }
        }
    }
}

#[cfg(test)]
pub use tests::*;
