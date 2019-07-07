mod ast;
mod compile;
mod rocketlang;
mod stack;
mod turing;

use turing::TuringMachine;

fn main() {
    let tm = TuringMachine::new(vec![]);
    tm.run("".to_string());
}
