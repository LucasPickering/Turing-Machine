use failure::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use tm::TuringMachine;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "tmcli",
    about = "Turing machines based on a simple two-variable stack machine."
)]
enum Opt {
    #[structopt(name = "run")]
    Execute {
        /// The file defining the Turing machine to run
        #[structopt(parse(from_os_str), long = "input", short = "i")]
        input_file: PathBuf,

        /// The input to pass to the machine for execution
        #[structopt(long = "tape", short = "t")]
        tape_input: String,
    },
    #[structopt(name = "compile")]
    Compile {
        /// The file defining the Turing machine to run
        #[structopt(parse(from_os_str), long = "input", short = "i")]
        input_file: PathBuf,

        /// The file to output Rocketlang code to
        #[structopt(parse(from_os_str), long = "output", short = "o")]
        output_file: PathBuf,
    },
}

fn tm_from_file(path: &PathBuf) -> Result<TuringMachine, Error> {
    let contents = fs::read_to_string(path)?;
    TuringMachine::from_json(&contents)
}

fn tm_to_file(tm: &TuringMachine, path: &PathBuf) -> Result<(), Error> {
    let mut contents = format!("{}", tm);
    fs::write(path, &mut contents)?;
    Ok(())
}

fn run(opt: Opt) -> Result<(), Error> {
    match opt {
        Opt::Execute {
            input_file,
            tape_input,
        } => {
            let tm = tm_from_file(&input_file)?;
            tm.run(tape_input)?;
            Ok(())
        }
        Opt::Compile {
            input_file,
            output_file,
        } => {
            let tm = tm_from_file(&input_file)?;
            tm_to_file(&tm, &output_file)
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    match run(opt) {
        Ok(()) => {}
        Err(error) => eprintln!("{:?}", error),
    }
}
