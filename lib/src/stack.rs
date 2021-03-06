use serde::Serialize;
use std::{
    fmt::{self, Display, Formatter},
    io::{self, Bytes, Read, Write},
};

/// The size of each register. For tape encoding, we're using 7 bits per char,
/// so this gives us 9 chars with one extra bit for the sign.
type Value = i64;

/// One step to run on the stack machine
#[derive(Clone, Debug, Serialize)]
pub enum SmInstruction {
    /// Reads one byte from input and sets the active variable to it. If there
    /// is nothing in the input to read, this does nothing.
    /// "Take the shot!"
    ReadToActive,

    /// Prints the current active variable to output, as a Unicode character.
    /// "I got it!"
    PrintActive,

    /// Prints both variables and the stack.
    /// "Sorry!"
    PrintState,

    /// Increments the active variable.
    /// "Wow!"
    IncrActive,

    /// Decrements the active variable.
    /// "Close one!"
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
    /// "No Problem."
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
    Comment(String),

    /// A comment that goes on the same line as an instruction
    ///
    /// Rocketlang doesn't support comments, so this is for debugging only.
    /// These comments will need to be stripped before passing the source to
    /// the Rocketlang interpreter.
    InlineComment(Box<Self>, String),

    /// Prints a string to stdout.
    DebugPrint(String, bool),
}

impl SmInstruction {
    /// Adds the given number of indentations.
    fn write_indents(f: &mut Formatter<'_>, indents: usize) -> fmt::Result {
        for _ in 0..indents {
            write!(f, "\t")?;
        }
        Ok(())
    }

    /// Formats a set of nested instructions. Wraps them in braces too, with
    /// the given label. Fun!
    fn fmt_nested(
        f: &mut Formatter<'_>,
        label: &str,
        instrs: &[Self],
        indents: usize,
    ) -> fmt::Result {
        writeln!(f, "{} {{", label)?;
        for instr in instrs {
            // Nested lines handle their own indentation
            instr.fmt_indented(f, indents + 1)?;
            writeln!(f)?;
        }
        Self::write_indents(f, indents)?;
        write!(f, "}}")
    }

    /// Formats this instruction with the specified number of indents.
    /// Tree recursion!
    fn fmt_indented(
        &self,
        f: &mut Formatter<'_>,
        indents: usize,
    ) -> fmt::Result {
        // Add the indentation for the first line
        Self::write_indents(f, indents)?;

        match self {
            SmInstruction::If(subinstrs) => {
                Self::fmt_nested(f, "If", &subinstrs, indents)
            }
            SmInstruction::While(subinstrs) => {
                Self::fmt_nested(f, "While", &subinstrs, indents)
            }
            SmInstruction::Comment(comment) => write!(f, "// {}", comment),
            SmInstruction::InlineComment(instr, comment) => {
                instr.fmt_indented(f, 0)?;
                write!(f, " // {}", comment)
            }
            // By default just use the debug name
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Display for SmInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

/// A direct equivalent of the rocketlang interpreter, equally as powerful.
/// All other machines must be built on top of this, so we know they can be
/// built in rocketlang.
pub struct StackMachine {
    active_var: Value,
    inactive_var: Value,
    stack: Vec<Value>,
    errors_enabled: bool,
}

impl StackMachine {
    /// Creates a new machine that reads from the given reader and writes to
    /// the given writer.
    pub fn new() -> Self {
        Self {
            active_var: 0,
            inactive_var: 0,
            stack: Vec::new(),
            errors_enabled: true,
        }
    }

    fn error_if_enabled(&self, error: &str) {
        if self.errors_enabled {
            panic!("$#@%! ({})", error)
        }
    }

    fn write_stack<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(
            format!(
                "Active: {}\nInactive: {}\n",
                self.active_var, self.inactive_var,
            )
            .as_bytes(),
        )?;
        writer.write_all("-----\n".as_bytes())?;
        for e in self.stack.iter().rev() {
            writer.write_all(&format!("- {}\n", e).as_bytes())?;
        }
        writer.write_all("\n".as_bytes())?;
        Ok(())
    }

    fn read_to_active<R: Read>(&mut self, reader: &mut Bytes<R>) {
        // Read one byte from stdin. If there is nothing to read, do
        // nothing.
        if let Some(res_b) = reader.next() {
            match res_b {
                Ok(b) => self.active_var = i64::from(b),
                Err(error) => {
                    self.error_if_enabled(&format!("Read error: {}", error));
                }
            }
        }
    }

    fn print_active<W: Write>(&self, writer: &mut W) {
        let to_write = &self.active_var.to_be_bytes()[7..];
        match writer.write_all(to_write) {
            Ok(()) => {}
            Err(error) => {
                self.error_if_enabled(&format!("Write error: {}", error));
            }
        }
    }

    fn print_state<W: Write>(&self, writer: &mut W) {
        match self.write_stack(writer) {
            Ok(()) => {}
            Err(error) => {
                self.error_if_enabled(&format!("Write error: {}", error));
            }
        }
    }

    fn incr(&mut self) {
        self.active_var += 1;
    }
    fn decr(&mut self) {
        self.active_var -= 1;
    }
    fn save_active(&mut self) {
        self.inactive_var = self.active_var;
    }
    fn swap(&mut self) {
        std::mem::swap(&mut self.active_var, &mut self.inactive_var);
    }
    fn push_zero(&mut self) {
        self.stack.push(0);
    }
    fn push_active(&mut self) {
        self.stack.push(self.active_var);
    }
    fn pop_to_active(&mut self) {
        match self.stack.pop() {
            Some(val) => {
                self.active_var = val;
            }
            None => {
                self.error_if_enabled("Pop on empty stack");
                // If we got here, we know errors are disabled
                self.active_var = 0;
            }
        }
    }
    fn toggle_errors(&mut self) {
        self.errors_enabled = !self.errors_enabled;
    }
    fn do_if<R: Read, W: Write>(
        &mut self,
        reader: &mut Bytes<R>,
        writer: &mut W,
        subinstrs: &[SmInstruction],
    ) {
        if self.active_var == self.inactive_var {
            for subinstr in subinstrs {
                self.run_instruction(reader, writer, subinstr)
            }
        }
    }
    fn do_while<R: Read, W: Write>(
        &mut self,
        reader: &mut Bytes<R>,
        writer: &mut W,
        subinstrs: &[SmInstruction],
    ) {
        while self.active_var > 0 {
            for subinstr in subinstrs {
                self.run_instruction(reader, writer, subinstr)
            }
        }
    }

    /// Runs a single instruction on this machine.
    fn run_instruction<R: Read, W: Write>(
        &mut self,
        reader: &mut Bytes<R>,
        writer: &mut W,
        instruction: &SmInstruction,
    ) {
        // These are all proxied to functions to make it easier to profile
        match instruction {
            SmInstruction::ReadToActive => self.read_to_active(reader),
            SmInstruction::PrintActive => {
                self.print_active(writer);
            }
            SmInstruction::PrintState => {
                self.print_state(writer);
            }
            SmInstruction::IncrActive => {
                self.incr();
            }
            SmInstruction::DecrActive => {
                self.decr();
            }
            SmInstruction::SaveActive => {
                self.save_active();
            }
            SmInstruction::Swap => {
                self.swap();
            }
            SmInstruction::PushZero => {
                self.push_zero();
            }
            SmInstruction::PushActive => {
                self.push_active();
            }
            SmInstruction::PopToActive => {
                self.pop_to_active();
            }
            SmInstruction::ToggleErrors => {
                self.toggle_errors();
            }
            SmInstruction::If(subinstrs) => {
                self.do_if(reader, writer, subinstrs)
            }
            SmInstruction::While(subinstrs) => {
                self.do_while(reader, writer, subinstrs);
            }
            SmInstruction::Comment(_) => {}
            SmInstruction::InlineComment(subinstr, _) => {
                self.run_instruction(reader, writer, subinstr)
            }
            SmInstruction::DebugPrint(msg, print_stack) => {
                println!("[DEBUG] {}", &msg);
                if *print_stack {
                    self.write_stack(&mut io::stdout()).unwrap();
                }
            }
        }
    }

    /// Runs all given instructions on this machine, using the given input
    /// and output.
    pub fn run<R: Read, W: Write>(
        &mut self,
        reader: R,
        writer: &mut W,
        instructions: &[SmInstruction],
    ) {
        let mut reader_bytes = reader.bytes();
        for instruction in instructions {
            self.run_instruction(&mut reader_bytes, writer, instruction)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SmInstruction::*, *};
    use std::io;

    fn run_machine_with_input<R: Read>(
        sm: &mut StackMachine,
        instructions: &[SmInstruction],
        input: R,
    ) {
        sm.run(input, &mut Vec::new(), instructions);
    }

    fn run_machine(sm: &mut StackMachine, instructions: &[SmInstruction]) {
        run_machine_with_input(sm, instructions, io::empty())
    }

    #[test]
    fn test_read_to_active() {
        let mut sm = StackMachine::new();
        run_machine_with_input(&mut sm, &[ReadToActive], &b"wee"[..]);
        assert_eq!(sm.active_var, 119);
    }

    #[test]
    fn test_incr_active() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[IncrActive]);
        assert_eq!(sm.active_var, 1);
    }

    #[test]
    fn test_decr_active() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[DecrActive]);
        assert_eq!(sm.active_var, -1);
    }

    #[test]
    fn test_save_active() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[IncrActive, SaveActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_swap() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[IncrActive, Swap]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_push_zero() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[IncrActive, PushZero]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[0]);
    }

    #[test]
    fn test_push_active() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[IncrActive, PushActive]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(&sm.stack, &[1]);
    }

    #[test]
    fn test_pop_to_active() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[IncrActive, PushZero, PopToActive]);
        assert_eq!(sm.active_var, 0);
        assert!(&sm.stack.is_empty());
    }

    #[test]
    #[should_panic(expected = "Pop on empty")]
    fn test_pop_to_active_on_empty_error() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[PopToActive]);
    }

    #[test]
    fn test_pop_to_active_on_empty_no_error() {
        let mut sm = StackMachine::new();
        run_machine(&mut sm, &[ToggleErrors, PopToActive]);
    }

    #[test]
    fn test_if_positive() {
        let mut sm = StackMachine::new();
        // If DOES run
        run_machine(&mut sm, &[If(vec![IncrActive, Swap])]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 1);
    }

    #[test]
    fn test_if_negative() {
        let mut sm = StackMachine::new();
        // If DOESN'T run
        run_machine(&mut sm, &[IncrActive, If(vec![Swap])]);
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 0);
    }

    #[test]
    fn test_while() {
        let mut sm = StackMachine::new();
        // If DOESN'T run
        run_machine(
            &mut sm,
            &[
                IncrActive,
                IncrActive,
                IncrActive,
                While(vec![PushZero, DecrActive]),
            ],
        );
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.stack, &[0, 0, 0]);
    }

    #[test]
    fn test_comment() {
        let mut sm = StackMachine::new();
        // Comment does nothing
        run_machine(&mut sm, &[Comment("Comment!".into())]);
        assert_eq!(sm.active_var, 0);
        assert_eq!(sm.inactive_var, 0);
        assert!(sm.stack.is_empty());
    }

    #[test]
    fn test_inline_comment() {
        let mut sm = StackMachine::new();
        // Comment does nothing
        run_machine(
            &mut sm,
            &[InlineComment(box IncrActive, "Comment!".into())],
        );
        assert_eq!(sm.active_var, 1);
        assert_eq!(sm.inactive_var, 0);
        assert!(sm.stack.is_empty());
    }
}
