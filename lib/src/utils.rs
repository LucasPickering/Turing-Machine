#[cfg(test)]
mod tests {
    use crate::error::CompilerResult;
    use std::fmt::Debug;

    /// Utility function for testing
    pub fn assert_compile_error<T: Debug>(
        msg: &str,
        result: CompilerResult<T>,
    ) {
        assert!(result.is_err());
        match result {
            Ok(_) => panic!("Expected Err but received Ok!"),
            Err(errors) => {
                let error_strs: Vec<String> =
                    errors.iter().map(|err| err.to_string()).collect();
                if !error_strs.iter().any(|error_str| error_str.contains(msg)) {
                    panic!(
                        "Expected one error of {:?} to contain substring \"{}\"",
                        error_strs, msg
                    );
                }
            }
        }
    }
}

#[cfg(test)]
pub use tests::*;
