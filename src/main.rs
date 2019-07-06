mod compile;
mod rocketlang;
mod stack;
mod turing;

use turing::{State, TuringMachine};

fn main() {
    let state = State {
        id: 0,
        transitions: vec![],
    };
    let tm = TuringMachine::new(&[state], 0);
    tm.run("".to_string());
}
