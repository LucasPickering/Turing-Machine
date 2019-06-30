mod stack;
mod turing;

use std::collections::HashMap;
use turing::TuringMachine;

fn main() {
    let tm = TuringMachine::new(HashMap::new());
    tm.run("".to_string());
}
