use std::path::PathBuf;
use structopt::StructOpt;
use tm::CompilerError;
use tm::TuringMachine;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "tmcli",
    about = "Turing machines based on a simple stack machine."
)]
enum Opt {
    #[structopt(name = "run")]
    Execute {
        /// The file defining the Turing machine to run
        #[structopt(parse(from_os_str))]
        input_file: PathBuf,

        /// The input to pass to the machine for execution
        #[structopt(long = "tape", short = "t")]
        tape_input: String,
    },
    #[structopt(name = "compile")]
    Compile {
        /// The file defining the Turing machine to run
        #[structopt(parse(from_os_str), long = "input", short)]
        input_file: PathBuf,

        /// The file to output Rocketlang code to
        #[structopt(parse(from_os_str), long = "output", short)]
        output_file: PathBuf,
    },
}

fn print_errors(msg: &str, errors: &[CompilerError]) {
    eprintln!("{}", msg);
    for error in errors {
        eprintln!("{}", error);
    }
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Execute {
            input_file,
            tape_input,
        } => match TuringMachine::from_file(&input_file) {
            Ok(tm) => match tm.run(tape_input) {
                Ok(()) => {}
                Err(errors) => print_errors("Error(s) in tape input:", &errors),
            },
            Err(errors) => print_errors("Error(s) compiling machine:", &errors),
        },
        Opt::Compile {
            input_file,
            output_file,
        } => match TuringMachine::from_file(&input_file) {
            Ok(tm) => {
                //TODO
            }
            Err(errors) => print_errors("Error(s) in tape input:", &errors),
        },
    }
}
