use std::collections::{HashMap, HashSet};
use std::iter::Iterator;

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
//#################################################################################################
//
//                                      Instruction Type
//
//#################################################################################################

#[derive(Copy, Clone, Debug)]
pub struct SingleInstruction<T> {
    pub(crate) target: T,
    pub(crate) gate_name: &'static str,
    pub(crate) control: Option<T>,
    pub(crate) reverse: bool,
}

pub(crate) type Instruction = SingleInstruction<Address>;

//#################################################################################################
//
//                                   Instruction chain trait
//
//#################################################################################################

pub trait InstructionChain<A> {
    fn push_instruction(
        &mut self, 
        target: A,
        gate_name: &'static str, 
        control: Option<A>,
        reverse: bool,
    );

    fn apply<C>(
        &mut self, 
        target: A,
        gate_name: &'static str, 
        control: C,
    ) -> &mut Self
    where
        C: Into<Option<A>>,
    {
        self.push_instruction(
            target,
            gate_name,
            control.into(),
            false,
        );

        self
    }

    fn apply_range<R, C>(
        &mut self, 
        targets: R,
        gate_name: &'static str, 
        control: C,
    ) -> &mut Self
    where  
        C: Into<Option<A>> + Copy,
        R: Iterator<Item = A>,
    {
        for target in targets {
            self.push_instruction(
                target,
                gate_name,
                control.into(),
                false,
            );
        }

        self
    }

    fn unapply<C>(
        &mut self, 
        target: A,
        gate_name: &'static str, 
        control: C,
    ) -> &mut Self
    where
        C: Into<Option<A>>,
    {
        self.push_instruction(
            target,
            gate_name,
            control.into(),
            true,
        );

        self
    }

    fn unapply_range<R, C>(
        &mut self, 
        targets: R,
        gate_name: &'static str, 
        control: C,
    ) -> &mut Self
    where  
        C: Into<Option<A>> + Copy,
        R: Iterator<Item = A>,
    {
        for target in targets {
            self.push_instruction(
                target,
                gate_name,
                control.into(),
                true,
            );
        }

        self
    }
}

//#################################################################################################
//
//                        SubRoutine & SubRoutineBuilder types
//
//#################################################################################################

#[derive(Debug)]
pub struct SubRoutine {
    variables: Box<[char]>,
    instructions: Box<[SingleInstruction<char>]>,
}

#[derive(Debug)]
pub struct SubRoutineBuilder<'a> {
    name: &'static str,
    variables: HashSet<char>,
    instructions: Vec<SingleInstruction<char>>,
    program: &'a mut ProgramBuilder<'a>,
    ended: bool,
}

impl<'a> SubRoutineBuilder<'a> {
    pub fn end(&mut self) -> &mut ProgramBuilder<'a> {
        self.ended = true;

        let variables = {
            let mut result = HashSet::with_capacity(0);
            std::mem::swap(&mut self.variables, &mut result);
            result.iter().map(|c| *c).collect()
        };

        let instructions = {
            let mut result = Vec::with_capacity(0);
            std::mem::swap(&mut self.instructions, &mut result);
            result.into()
        };

        self.program.subroutines.insert(self.name, SubRoutine {
            variables,
            instructions,
        });

        self.program
    }
}

impl InstructionChain<char> for SubRoutineBuilder<'_> {
    fn push_instruction(
        &mut self,
        target: char,
        gate_name: &'static str, 
        control: Option<char>,
        reverse: bool,
    ) {
        assert!(
            !self.ended, 
            "SubRoutine has already been ended, cannot add more gates"
        );
        assert!(
            self.variables.contains(&target),
            "Target's variable name '{}' was not declared", 
            target,
        );
        assert!(
            self.program.computer.gates.contains_key(gate_name),
            "No gate associated to the name \"{}\"", 
            gate_name,
        );
        if let Some(control) = control {
            assert!(
                self.variables.contains(&control),
                "Control's variable name '{}' was not declared", 
                control,
            );
        }

        self.instructions.push(SingleInstruction {
            target,
            gate_name,
            control,
            reverse,
        });
    }
}

//#################################################################################################
//
//                                       Program Builder
//
//#################################################################################################

/// A builder for the `Program` struct.
#[derive(Debug)]
pub struct ProgramBuilder<'a> {
    initial_state: usize,
    instructions: Vec<Instruction>,
    subroutines: HashMap<&'static str, SubRoutine>,
    computer: &'a Computer,
    measured: bool,
}

impl<'a> ProgramBuilder<'a> {
    pub(crate) fn new(computer: &'a Computer, initial_state: &str) -> ProgramBuilder<'a> {
        let initial_state = parse_state(computer, initial_state);

        let instructions = Vec::new();

        let subroutines = HashMap::new();

        let measured = false;

        ProgramBuilder {
            initial_state,
            instructions,
            subroutines,
            computer,
            measured,
        }
    }

    pub fn new_subroutine<V>(&'a mut self, name: &'static str, variables: V) -> SubRoutineBuilder where
        V: Iterator<Item = char>,
    {
        assert!(
            !self.subroutines.contains_key(name),
            "There already exists a SubRoutine name \"{}\"",
            name,
        );

        let variables = variables.collect();

        let instructions = Vec::new();

        let program = self;

        let ended = false;

        SubRoutineBuilder {
            name,
            variables,
            instructions,
            program,
            ended,
        }
    }

    pub fn measure(&mut self, samples: usize) -> Program {
        assert!(samples != 0, "Samples count cannot be 0");

        self.measured = true;

        let initial_state = self.initial_state;

        let instructions = {
            let mut result = Vec::with_capacity(0);
            std::mem::swap(&mut self.instructions, &mut result);
            result.into()
        };

        let samples = samples;

        Program {
            initial_state,
            instructions,
            samples,
        }
    } 
}

impl InstructionChain<Address> for ProgramBuilder<'_> {
    fn push_instruction(
        &mut self,
        target: Address,
        gate_name: &'static str, 
        control: Option<Address>,
        reverse: bool,
    ) {
        assert!(
            !self.measured, 
            "State has already been measured, cannot add more gates"
        );
        assert!(
            target < self.computer.size,
            "Target's address (#{}) is out of the {}-sized register", 
            target,
            self.computer.size,
        );
        assert!(
            self.computer.gates.contains_key(gate_name),
            "No gate associated to the name \"{}\"", 
            gate_name,
        );
        if let Some(control) = control {
            assert!(
                control < self.computer.size,
                "Control's address (#{}) is out of the {}-sized register", 
                control,
                self.computer.size,
            );
        }

        self.instructions.push(SingleInstruction {
            target,
            gate_name,
            control,
            reverse,
        });
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
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}