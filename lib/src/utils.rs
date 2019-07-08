#[cfg(test)]
mod tests {
    use failure::Error;
    use std::fmt::Debug;

    /// Utility function for testing
    pub fn assert_compile_error<T: Debug>(msg: &str, result: Result<T, Error>) {
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
