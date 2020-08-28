use crate::computer::Address;

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
pub struct ProgramBuilder {
    initial_state: usize,
    instructions: Vec<Instruction>,
}

impl ProgramBuilder {
    pub fn apply(
        &mut self, 
        target: Address,
        gate_name: &'static str, 
        control: Option<Address>
    ) -> &mut ProgramBuilder
    {
        self.instructions.push(Instruction {
            target,
            gate_name,
            control,
        });
        self
    }

    pub fn apply_range(
        &mut self, 
        targets: std::ops::Range<Address>,
        gate_name: &'static str, 
        control: Option<Address>
    ) -> &mut ProgramBuilder
    {
        for target in targets {
            self.instructions.push(Instruction {
                target,
                gate_name,
                control,
            });
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
    pub fn new(initial_state: usize) -> ProgramBuilder {
        let instructions = Vec::new();

        ProgramBuilder {
            initial_state, 
            instructions,
        }
    }
}