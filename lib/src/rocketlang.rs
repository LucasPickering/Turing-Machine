use crate::stack::SmInstruction;
use itertools::Itertools;

trait ToRocketlang {
    fn to_rocketlang(&self) -> String;
}

impl<R: ToRocketlang> ToRocketlang for Vec<R> {
    fn to_rocketlang(&self) -> String {
        self.iter().map(R::to_rocketlang).join("\n")
    }
}

impl ToRocketlang for SmInstruction {
    fn to_rocketlang(&self) -> String {
        match self {
            SmInstruction::ReadToActive => "Take the shot!".to_owned(),
            SmInstruction::PrintActive => "I got it!".to_owned(),
            SmInstruction::PrintState => "Sorry!".to_owned(),
            SmInstruction::IncrActive => "Wow!".to_owned(),
            SmInstruction::DecrActive => "Close one!".to_owned(),
            SmInstruction::SaveActive => "Whoops...".to_owned(),
            SmInstruction::Swap => "OMG!".to_owned(),
            SmInstruction::PushZero => "Noooo!".to_owned(),
            SmInstruction::PushActive => "Defending...".to_owned(),
            SmInstruction::PopToActive => "Centering...".to_owned(),
            SmInstruction::ToggleErrors => "No Problem.".to_owned(),
            SmInstruction::If(subinstrs) => format!(
                "Nice shot!\n{}\nWhat a save!",
                subinstrs.to_rocketlang()
            ),
            SmInstruction::While(subinstrs) => {
                format!("Great pass!\n{}\nThanks!", subinstrs.to_rocketlang())
            }
            // Rocketlang has no syntax for comments so we can't do anything
            SmInstruction::Comment(_) => String::new(),
            SmInstruction::InlineComment(subinstr, _) => {
                subinstr.to_rocketlang()
            }
            SmInstruction::DebugPrint(_,_) => String::new(),
        }
    }
}
