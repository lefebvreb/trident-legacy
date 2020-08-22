use crate::computer::Computer;
use crate::error::{Error, QRustResult};

#[derive(Copy, Clone)]
enum ApplyGate {
    Unitary {
        gate_id: char,
        target: u16,
    },
    Controlled {
        gate_id: char,
        target: u16,
        control: u16,
    },
}

pub struct ProgramBuilder<'a> {
    computer: &'a Computer,
    instructions: Vec<ApplyGate>,
}

pub struct Program<'a> {
    computer: &'a Computer,
    instructions: Box<[ApplyGate]>,
}

impl<'a> Program<'a> {
    pub fn new(computer: &'a Computer) -> ProgramBuilder<'a> {
        let instructions = Vec::new();
        
        ProgramBuilder {
            computer,
            instructions,
        }
    }

    pub fn run(&self) {

    }
}

impl ProgramBuilder<'_> {
    pub fn apply_gate(&mut self, gate_id: char, target: u16) -> &mut Self {
        self.instructions.push(ApplyGate::Unitary {
            gate_id,
            target,
        });
        self
    }

    pub fn apply_controlled_gate(&mut self, gate_id: char, target: u16, control: u16) -> &mut Self {
        self.instructions.push(ApplyGate::Controlled {
            gate_id,
            target,
            control,
        });
        self
    }

    pub fn build(&self) -> QRustResult<Program> {
        let computer = self.computer;
        
        let instructions = {
            let mut res = Vec::with_capacity(self.instructions.len());

            for instruction in self.instructions.iter() {
                match instruction {
                    ApplyGate::Unitary {gate_id, target} => {
                        if !self.computer.contains_gate(gate_id) {
                            return Err(Error::new());
                        } else if *target < self.computer.size() {
                            return Err(Error::new());
                        }
                    },
                    ApplyGate::Controlled {gate_id, target, control} => {
                        if !self.computer.contains_gate(gate_id) {
                            return Err(Error::new());
                        } else if *target < self.computer.size() {
                            return Err(Error::new());
                        } else if *control < self.computer.size() {
                            return Err(Error::new());
                        }
                    },
                }
                res.push(*instruction);
            }
            
            res.into()
        };

        Ok(Program {
            computer,
            instructions,
        })
    }
}