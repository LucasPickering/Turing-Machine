use std::io;
use std::io::Bytes;
use std::io::Read;
use std::io::Stdin;
use std::io::Stdout;
use std::io::Write;

type Value = i64;

/// One step to run on the stack machine
pub enum SmAction {
    /// Reads one byte from stdin and sets the active variable to it.
    /// "Take the shot!"
    ReadToActive,

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
    Push0,

    /// Pushes the active value onto the top of the stack
    /// "Defending..."
    PushActive,

    /// Pops the value at the top of the stack and sets the active var to it.
    /// If the stack is empty, then this `panic!`s.
    /// "Centering..."
    PopToActive,

    /// Runs all given steps, in order, iff active_var == inactive_var.
    /// "Nice shot!" and "What a save!" - we could require the user to end with
    /// an EndIf action  to preserve better correlation with rocketlang,
    /// but I think this shortcut is okay to take.
    If(Vec<Self>),

    /// Runs all given steps, in order, while active_var > 0.
    /// "Great pass!" and "Thanks!" - we could require the user to end  with
    /// an EndWhile action to preserve better correlation with rocketlang,
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
            stack: vec![],
            reader: reader.bytes(),
            writer,
        }
    }

    pub fn get_active(&self) -> Value {
        self.active_var
    }

    /// Runs a single action on this machine.
    fn run_action(&mut self, action: &SmAction) {
        match action {
            SmAction::ReadToActive => {
                // Read one byte from stdin
                // TODO: This is bad
                self.active_var =
                    self.reader.next().and_then(|result| result.ok()).unwrap()
                        as i64;
            }
            SmAction::IncrActive => {
                self.active_var += 1;
            }
            SmAction::DecrActive => {
                self.active_var -= 1;
            }
            SmAction::SaveActive => {
                self.inactive_var = self.active_var;
            }
            SmAction::Swap => {
                std::mem::swap(&mut self.active_var, &mut self.inactive_var);
            }
            SmAction::Push0 => {
                self.stack.push(0);
            }
            SmAction::PushActive => {
                self.stack.push(self.active_var);
            }
            SmAction::PopToActive => match self.stack.pop() {
                Some(val) => {
                    self.active_var = val;
                }
                // Runtime errors: easiest punt on the planet
                None => panic!("Pop on empty stack!"),
            },
            SmAction::If(subactions) => {
                if self.active_var == self.inactive_var {
                    for subaction in subactions {
                        self.run_action(subaction)
                    }
                }
            }
            SmAction::While(subactions) => {
                while self.active_var > 0 {
                    for subaction in subactions {
                        self.run_action(subaction)
                    }
                }
            }
        }
    }

    /// Runs all given actions on this machine.
    pub fn run(&mut self, actions: &[SmAction]) {
        for action in actions {
            self.run_action(action)
        }
    }
}

impl StackMachine<Stdin, Stdout> {
    /// Creates a new machine that reads from stdin and writes to stdout.
    pub fn new_std() -> Self {
        Self::new(io::stdin(), io::stdout())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::IncrActive]);
        assert_eq!(sm.get_active(), 1);
    }

    #[test]
    fn test_read_to_active() {
        let mut sm = StackMachine::new(&b"\x09"[..], Vec::new());
        sm.run(&[SmAction::ReadToActive]);
        assert_eq!(sm.active_var, 9);
    }

    #[test]
    fn test_incr_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::IncrActive]);
        assert_eq!(sm.active_var, 1);
    }

    #[test]
    fn test_decr_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::DecrActive]);
        assert_eq!(sm.active_var, -1);
    }

    #[test]
    fn test_save_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::IncrActive, SmAction::SaveActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_swap() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::IncrActive, SmAction::Swap]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_push_0() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::IncrActive, SmAction::Push0]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[0]);
    }

    #[test]
    fn test_push_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::IncrActive, SmAction::PushActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[1]);
    }

    #[test]
    fn test_pop_to_active() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::IncrActive, SmAction::Push0, SmAction::PopToActive]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(&sm.stack, &[]);
    }

    #[test]
    #[should_panic(expected = "Pop on empty")]
    fn test_pop_to_active_on_empty() {
        let mut sm = StackMachine::new_std();
        sm.run(&[SmAction::PopToActive]);
    }

    #[test]
    fn test_if_positive() {
        let mut sm = StackMachine::new_std();
        // If DOES run
        sm.run(&[SmAction::If(vec![SmAction::IncrActive, SmAction::Swap])]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_if_negative() {
        let mut sm = StackMachine::new_std();
        // If DOESN'T run
        sm.run(&[SmAction::IncrActive, SmAction::If(vec![SmAction::Swap])]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 0);
    }

    #[test]
    fn test_while() {
        let mut sm = StackMachine::new_std();
        // If DOESN'T run
        sm.run(&[
            SmAction::IncrActive,
            SmAction::IncrActive,
            SmAction::IncrActive,
            SmAction::While(vec![SmAction::Push0, SmAction::DecrActive]),
        ]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.stack, &[0, 0, 0]);
    }
}
