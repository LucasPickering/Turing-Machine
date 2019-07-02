use std::io::Bytes;
use std::io::Read;
use std::io::Write;

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
    /// If the stack is empty, then this `panic!`s.
    /// "Centering..."
    PopToActive,

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
}

/// A direct equivalent of the rocketlang interpreter, equally as powerful.
/// All other machines must be built on top of this, so we know they can be
/// built in rocketlang.
pub struct StackMachine<R: Read, W: Write> {
    active_var: Value,
    inactive_var: Value,
    stack: Vec<Value>,
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
                    self.reader.next().and_then(|result| result.ok()).unwrap(),
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
                None => panic!("Pop on empty stack!"),
            },
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
    use super::*;

    #[test]
    fn test_get_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::IncrActive]);
        assert_eq!(sm.get_active(), 1);
    }

    #[test]
    fn test_read_to_active() {
        let mut sm = StackMachine::new(&b"\x09"[..], Vec::new());
        sm.run(&[SmInstruction::ReadToActive]);
        assert_eq!(sm.active_var, 9);
    }

    #[test]
    fn test_incr_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::IncrActive]);
        assert_eq!(sm.active_var, 1);
    }

    #[test]
    fn test_decr_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::DecrActive]);
        assert_eq!(sm.active_var, -1);
    }

    #[test]
    fn test_save_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::IncrActive, SmInstruction::SaveActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_swap() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::IncrActive, SmInstruction::Swap]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_push_zero() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::IncrActive, SmInstruction::PushZero]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[0]);
    }

    #[test]
    fn test_push_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::IncrActive, SmInstruction::PushActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[1]);
    }

    #[test]
    fn test_pop_to_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[
            SmInstruction::IncrActive,
            SmInstruction::PushZero,
            SmInstruction::PopToActive,
        ]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(&sm.stack, &[]);
    }

    #[test]
    #[should_panic(expected = "Pop on empty")]
    fn test_pop_to_active_on_empty() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmInstruction::PopToActive]);
    }

    #[test]
    fn test_if_positive() {
        let mut sm = StackMachine::new_std();
        // If DOES run
        sm.run(&[SmInstruction::If(vec![
            SmInstruction::IncrActive,
            SmInstruction::Swap,
        ])]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_if_negative() {
        let mut sm = StackMachine::new_std();
        // If DOESN'T run
        sm.run(&[
            SmInstruction::IncrActive,
            SmInstruction::If(vec![SmInstruction::Swap]),
        ]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 0);
    }

    #[test]
    fn test_while() {
        let mut sm = StackMachine::new_std();
        // If DOESN'T run
        sm.run(&[
            SmInstruction::IncrActive,
            SmInstruction::IncrActive,
            SmInstruction::IncrActive,
            SmInstruction::While(vec![
                SmInstruction::PushZero,
                SmInstruction::DecrActive,
            ]),
        ]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.stack, &[0, 0, 0]);
    }
}
