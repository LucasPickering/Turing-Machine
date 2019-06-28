use crate::stack::StackMachine;

type Char = u8; // Will be truncated to 7 bits - ASCII only

enum TmAction {
    Left,
    Right,
    Write(Char),
}

struct TuringMachine(StackMachine);
