use crate::computer::Address;

//#################################################################################################
//
//                                       Instruction
//
//#################################################################################################

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

pub struct ProgramBuilder {
    initial_state: usize,
    instructions: Vec<Instruction>,
}

impl ProgramBuilder {
    pub fn apply(mut self, target: Address, gate_name: &'static str) -> ProgramBuilder {
        let control = None;
        self.instructions.push(Instruction {
            target,
            gate_name,
            control,
        });

        self
    }

    pub fn apply_controlled(mut self, target: Address, gate_name: &'static str, control: Address) -> ProgramBuilder {
        let control = Some(control);

        self.instructions.push(Instruction {
            target,
            gate_name,
            control,
        });
        self
    }

    pub fn measure(self, samples: u16) -> Program {
        if samples == 0 {
            panic!("Samples count cannot be null")
        }

        let initial_state = self.initial_state;

        let instructions = self.instructions.into();

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

pub struct Program {
    pub(crate) initial_state: usize,
    pub(crate) instructions: Box<[Instruction]>,
    pub(crate) samples: u16,
}

impl Program {
    pub fn new(initial_state: usize) -> ProgramBuilder {
        let instructions = Vec::new();

        ProgramBuilder {
            initial_state, 
            instructions,
        }
    }
}