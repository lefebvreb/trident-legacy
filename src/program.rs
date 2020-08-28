use crate::computer::{Address, Computer};

//#################################################################################################
//
//                                     Helper functions
//
//#################################################################################################

fn parse_state(computer: &Computer, state: &str) -> usize {
    if state.len() != computer.size as usize + 2 {
        panic!(
            "The given initial state \"{}\" is invalid, it must match \"|[01]{{{}}}>\": the length is invalid", 
            state,
            computer.size,
        );
    }

    let mut chars =  state.chars();

    if chars.next().unwrap() != '|' {
        panic!(
            "The given initial state \"{}\" is invalid, it must match \"|[01]{{{}}}>\": does not begin with a '|'", 
            state,
            computer.size,
        );
    }

    let mut result = 0;

    for i in 0..computer.size {
        let digit = chars.next().unwrap();

        match digit {
            '0' => (),
            '1' => result |= 1usize << i,
            _ => panic!(
                "The given initial state \"{}\" is invalid, it must match \"|[01]{{{}}}>\": digit #{} is invalid", 
                state,
                computer.size,
                i+1,
            ),
        }
    }

    if chars.next().unwrap() != '>' {
        panic!(
            "The given initial state \"{}\" is invalid, it must match \"|[01]{{{}}}>\": does not end with a '>'", 
            state,
            computer.size,
        );
    }

   result
}

fn check_instruction(computer: &Computer, instruction: Instruction) {
    if instruction.target >= computer.size {
        panic!(
            "Target's address (#{}) is out of the {}-sized register", 
            instruction.target,
            computer.size,
        );
    } else if !computer.gates.contains_key(instruction.gate_name) {
        panic!(
            "No gate associated to the name \"{}\"", 
            instruction.gate_name,
        );
    } else if let Some(control) = instruction.control {
        if control >= computer.size {
            panic!(
                "Control's address (#{}) is out of the {}-sized register", 
                control,
                computer.size,
            );
        }
    }
}

//#################################################################################################
//
//                                       Instruction
//
//#################################################################################################

#[derive(Copy, Clone, Debug)]
pub(crate) struct Instruction {
    pub(crate) target: Address,
    pub(crate) gate_name: &'static str,
    pub(crate) control: Option<Address>,
}

//#################################################################################################
//
//                                       Program Builder
//
//#################################################################################################

#[derive(Debug)]
pub struct ProgramBuilder<'a> {
    initial_state: usize,
    instructions: Vec<Instruction>,
    computer: &'a Computer,
}

impl<'a> ProgramBuilder<'a> {
    pub fn apply(
        &'a mut self, 
        target: Address,
        gate_name: &'static str, 
        control: Option<Address>
    ) -> &'a mut ProgramBuilder {
        let instruction = Instruction {
            target,
            gate_name,
            control,
        };
        check_instruction(self.computer, instruction);
        self.instructions.push(instruction);

        self
    }

    pub fn apply_range(
        &'a mut self, 
        targets: std::ops::Range<Address>,
        gate_name: &'static str, 
        control: Option<Address>
    ) -> &'a mut ProgramBuilder {
        for target in targets {
            let instruction = Instruction {
                target,
                gate_name,
                control,
            };
            check_instruction(self.computer, instruction);
            self.instructions.push(instruction);
        }        

        self
    }

    pub fn measure(&self, samples: usize) -> Program {
        if samples == 0 {
            panic!("Samples count cannot be 0")
        }

        let initial_state = self.initial_state;
        let instructions = self.instructions.clone().into();
        let samples = samples;

        Program {
            initial_state,
            instructions,
            samples,
        }
    } 
}

//#################################################################################################
//
//                                           Program
//
//#################################################################################################

#[derive(Debug)]
pub struct Program {
    pub(crate) initial_state: usize,
    pub(crate) instructions: Box<[Instruction]>,
    pub(crate) samples: usize,
}

impl Program {
    pub fn new<'a>(computer: &'a Computer, initial_state: &str) -> ProgramBuilder<'a> {
        let initial_state = parse_state(computer, initial_state);

        let instructions = Vec::new();

        ProgramBuilder {
            initial_state,
            instructions,
            computer,
        }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}