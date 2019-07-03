use std::io::{Bytes, Read, Write};

type Value = i64;

/// One step to run on the stack machine
#[derive(Clone, Debug)]
pub enum SmInstruction {
    /// Reads one byte from input and sets the active variable to it.
    /// "Take the shot!"
    ReadToActive,

    /// Prints the current active variable to output, as a Unicode character.
    /// "I got it!"
    PrintActive,

    /// Increments the active variable.
    /// "Wow!"
    IncrActive,

    /// Decrements the active variable.
    /// "Close One!"
    DecrActive,

    /// Copies the active variable to the inactive variable.
    /// "Whoops..."
    SaveActive,

    /// Swaps the active and inactive values.
    /// "OMG!"
    Swap,

    /// Pushes the value 0 onto the stack.
    /// "Noooo!"
    PushZero,

    /// Pushes the active value onto the top of the stack
    /// "Defending..."
    PushActive,

    /// Pops the value at the top of the stack and sets the active var to it.
    /// If the stack is empty: If errors are enabled then this `panic!`s. If
    /// not, this pops 0.
    /// "Centering..."
    PopToActive,

    /// Toggles error handling. When enabled, errors (e.g. popping an empty
    /// stack) generate a panic!. When disabled, no errors (e.g. just pop 0).
    ToggleErrors,

    /// Runs all given steps, in order, iff active_var == inactive_var.
    /// "Nice shot!" and "What a save!" - we could require the user to end with
    /// an EndIf instruction to preserve better correlation with rocketlang,
    /// but I think this shortcut is okay to take.
    If(Vec<Self>),

    /// Runs all given steps, in order, while active_var > 0.
    /// "Great pass!" and "Thanks!" - we could require the user to end  with
    /// an EndWhile instruction to preserve better correlation with rocketlang,
    /// but I think this shortcut is okay to take.
    While(Vec<Self>),

    /// A standalone comment.
    ///
    /// Rocketlang doesn't support comments, so this is for debugging only.
    /// These comments will need to be stripped before passing the source to
    /// the Rocketlang interpreter.
    Comment(&'static str),

    /// A comment that goes on the same line as an instruction
    ///
    /// Rocketlang doesn't support comments, so this is for debugging only.
    /// These comments will need to be stripped before passing the source to
    /// the Rocketlang interpreter.
    InlineComment(Box<Self>, &'static str),
}

/// A direct equivalent of the rocketlang interpreter, equally as powerful.
/// All other machines must be built on top of this, so we know they can be
/// built in rocketlang.
pub struct StackMachine<R: Read, W: Write> {
    active_var: Value,
    inactive_var: Value,
    stack: Vec<Value>,
    errors_enabled: bool,
    reader: Bytes<R>,
    writer: W,
}

impl<R: Read, W: Write> StackMachine<R, W> {
    /// Creates a new machine that reads from the given reader and writes to
    /// the given writer.
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            active_var: 0,
            inactive_var: 0,
            stack: Vec::new(),
            errors_enabled: true,
            reader: reader.bytes(),
            writer,
        }
    }

    pub fn get_active(&self) -> Value {
        self.active_var
    }

    /// Runs a single instruction on this machine.
    fn run_instruction(&mut self, instruction: &SmInstruction) {
        match instruction {
            SmInstruction::ReadToActive => {
                // Read one byte from stdin
                // TODO error handling
                self.active_var = i64::from(
                    self.reader
                        .next()
                        .and_then(std::result::Result::ok)
                        .unwrap(),
                );
            }
            SmInstruction::PrintActive => {
                // TODO error handling
                self.writer
                    // Write the lowest 4 bytes, to represent a Unicode char
                    .write_all(&self.active_var.to_be_bytes()[4..])
                    .unwrap();
            }
            SmInstruction::IncrActive => {
                self.active_var += 1;
            }
            SmInstruction::DecrActive => {
                self.active_var -= 1;
            }
            SmInstruction::SaveActive => {
                self.inactive_var = self.active_var;
            }
            SmInstruction::Swap => {
                std::mem::swap(&mut self.active_var, &mut self.inactive_var);
            }
            SmInstruction::PushZero => {
                self.stack.push(0);
            }
            SmInstruction::PushActive => {
                self.stack.push(self.active_var);
            }
            SmInstruction::PopToActive => match self.stack.pop() {
                Some(val) => {
                    self.active_var = val;
                }
                // TODO error handling
                None => {
                    if self.errors_enabled {
                        panic!("Pop on empty stack!")
                    } else {
                        self.active_var = 0;
                    }
                }
            },
            SmInstruction::ToggleErrors => {
                self.errors_enabled = !self.errors_enabled;
            }
            SmInstruction::If(subinstrs) => {
                if self.active_var == self.inactive_var {
                    for subinstr in subinstrs {
                        self.run_instruction(subinstr)
                    }
                }
            }
            SmInstruction::While(subinstrs) => {
                while self.active_var > 0 {
                    for subinstr in subinstrs {
                        self.run_instruction(subinstr)
                    }
                }
            }
            SmInstruction::Comment(_) => {}
            SmInstruction::InlineComment(subinstr, _) => {
                self.run_instruction(subinstr)
            }
        }
    }

    /// Runs all given instructions on this machine.
    pub fn run(&mut self, instructions: &[SmInstruction]) {
        for instruction in instructions {
            self.run_instruction(instruction)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SmInstruction::*, *};

    fn make_sm() -> StackMachine<&'static [u8], Vec<u8>> {
        StackMachine::new(b"", Vec::new())
    }

    #[test]
    fn test_get_active() {
        let mut sm = make_sm();
        sm.run(&[IncrActive]);
        assert_eq!(sm.get_active(), 1);
    }

    #[test]
    fn test_read_to_active() {
        let mut sm: StackMachine<&'static [u8], Vec<u8>> =
            StackMachine::new(b"\x09", Vec::new());
        sm.run(&[ReadToActive]);
        assert_eq!(sm.active_var, 9);
    }

    #[test]
    fn test_incr_active() {
        let mut sm = make_sm();
        sm.run(&[IncrActive]);
        assert_eq!(sm.active_var, 1);
    }

    #[test]
    fn test_decr_active() {
        let mut sm = make_sm();
        sm.run(&[DecrActive]);
        assert_eq!(sm.active_var, -1);
    }

    #[test]
    fn test_save_active() {
        let mut sm = make_sm();
        sm.run(&[IncrActive, SaveActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_swap() {
        let mut sm = make_sm();
        sm.run(&[IncrActive, Swap]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_push_zero() {
        let mut sm = make_sm();
        sm.run(&[IncrActive, PushZero]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[0]);
    }

    #[test]
    fn test_push_active() {
        let mut sm = make_sm();
        sm.run(&[IncrActive, PushActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[1]);
    }

    #[test]
    fn test_pop_to_active() {
        let mut sm = make_sm();
        sm.run(&[IncrActive, PushZero, PopToActive]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(&sm.stack, &[]);
    }

    #[test]
    #[should_panic(expected = "Pop on empty")]
    fn test_pop_to_active_on_empty_error() {
        let mut sm = make_sm();
        sm.run(&[PopToActive]);
    }

    #[test]
    fn test_pop_to_active_on_empty_no_error() {
        let mut sm = make_sm();
        sm.run(&[ToggleErrors, PopToActive]);
    }

    #[test]
    fn test_if_positive() {
        let mut sm = make_sm();
        // If DOES run
        sm.run(&[If(vec![IncrActive, Swap])]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_if_negative() {
        let mut sm = make_sm();
        // If DOESN'T run
        sm.run(&[IncrActive, If(vec![Swap])]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 0);
    }

    #[test]
    fn test_while() {
        let mut sm = make_sm();
        // If DOESN'T run
        sm.run(&[
            IncrActive,
            IncrActive,
            IncrActive,
            While(vec![PushZero, DecrActive]),
        ]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.stack, &[0, 0, 0]);
    }

    #[test]
    fn test_comment() {
        let mut sm = make_sm();
        // Comment does nothing
        sm.run(&[Comment("Comment!")]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 0);
        assert!(sm.stack.is_empty());
    }

    #[test]
    fn test_inline_comment() {
        let mut sm = make_sm();
        // Comment does nothing
        sm.run(&[InlineComment(Box::new(IncrActive), "Comment!")]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 0);
        assert!(sm.stack.is_empty());
    }
}
