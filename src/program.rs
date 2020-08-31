use std::collections::{HashMap, HashSet};
use std::iter::IntoIterator;
use std::fmt;
use std::mem::swap;

use crate::computer::{Address, Computer};

//#################################################################################################
//
//                                     Helper functions
//
//#################################################################################################

// Parse a state from a &str. The regex is |[01]{size}> where size is the computer's size.
fn parse_state(computer: &Computer, state: &str) -> usize {
    assert!(
        state.len() == computer.size as usize + 2,
        "The given initial state \"{}\" is invalid, it must match \"|[01]{{{}}}>\": the length is invalid", 
        state,
        computer.size,
    );

    let mut chars =  state.chars();

    assert!(
        chars.next().unwrap() == '|',
        "The given initial state \"{}\" is invalid, it must match \"|[01]{{{}}}>\": does not begin with a '|'", 
        state,
        computer.size,
    );

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

    assert!(
        chars.next().unwrap() == '>',
        "The given initial state \"{}\" is invalid, it must match \"|[01]{{{}}}>\": does not end with a '>'", 
        state,
        computer.size,
    );

   result
}
//#################################################################################################
//
//                                      Instruction Type
//
//#################################################################################################

#[derive(Copy, Clone, Debug)]
pub(crate) struct SingleInstruction<T> {
    pub(crate) gate_name: &'static str,
    pub(crate) target: T,
    pub(crate) control: Option<T>,
    pub(crate) reverse: bool,
}

pub(crate) type Instruction = SingleInstruction<Address>;

//#################################################################################################
//
//                             InstructionChainInternals trait
//
//#################################################################################################

mod private {
    pub trait InstructionChainInternals<A>
    where
        A: Copy
    {
        fn push_instruction(
            &mut self, 
            gate_name: &'static str, 
            target: A,
            control: Option<A>,
            reverse: bool,
        );
    
        fn get_subroutine(
            &self,
            subroutine_name: &str,
        ) -> Option<&super::SubRoutine>;
    }
}

//#################################################################################################
//
//                                  InstructionChain trait
//
//#################################################################################################

pub trait InstructionChain<A>: private::InstructionChainInternals<A>
where
    A: Copy
{
    fn apply<C>(
        &mut self, 
        gate_name: &'static str, 
        target: A,
        control: C,
    ) -> &mut Self
    where
        C: Into<Option<A>>,
    {
        self.push_instruction(
            gate_name,
            target,
            control.into(),
            false,
        );

        self
    }

    fn apply_iter<R, C>(
        &mut self, 
        gate_name: &'static str,
        targets: R,        
        control: C,
    ) -> &mut Self
    where  
        R: IntoIterator<Item = A>,
        C: Into<Option<A>> + Copy,
    {
        for target in targets {
            self.push_instruction(
                gate_name,
                target,
                control.into(),
                false,
            );
        }

        self
    }

    fn unapply<C>(
        &mut self, 
        gate_name: &'static str, 
        target: A,        
        control: C,
    ) -> &mut Self
    where
        C: Into<Option<A>>,
    {
        self.push_instruction(
            gate_name,
            target,
            control.into(),
            true,
        );

        self
    }

    fn unapply_iter<R, C>(
        &mut self, 
        gate_name: &'static str, 
        targets: R,
        control: C,
    ) -> &mut Self
    where  
        R: IntoIterator<Item = A>,
        C: Into<Option<A>> + Copy,
    {
        for target in targets {
            self.push_instruction(
                gate_name,
                target,
                control.into(),
                true,
            );
        }

        self
    }

    fn call<V>(
        &mut self,
        subroutine_name: &'static str,
        arguments: V,
    ) -> &mut Self 
    where
        V: Iterator<Item = (char, A)>
    {
        // Yep, a pointer. Seriously, f*** borrowck.
        let subroutine: *const SubRoutine = self.get_subroutine(subroutine_name)
            .expect(format!(
                "There are no subroutines named \"{}\"",
                subroutine_name,
            ).as_str());

        let map = {
            let mut result = HashMap::with_capacity(arguments.size_hint().0);
            for (variable, argument) in arguments.take(unsafe {(*subroutine).variables.len()}) {
                assert!(
                    unsafe {(*subroutine).variables.contains(&variable)}, 
                    "No variable named '{}' in subroutine named \"{}\"",
                    variable,
                    subroutine_name,
                );
                result.insert(variable, argument);
            }
            result
        };

        for instruction in unsafe {(*subroutine).instructions.iter()} {
            let target = *map.get(&instruction.target)
                .expect(format!(
                    "Can't match variable '{}' to a value: value not specified",
                    instruction.target,
                ).as_str());

            let control = instruction.control.map(|c| 
                *map.get(&c)
                    .expect(format!(
                        "Can't match variable '{}' to a value: value not specified",
                        c,
                    ).as_str())
            );

            self.push_instruction(
                instruction.gate_name,
                target,
                control,
                instruction.reverse,
            );
        }

        self
    }

    fn uncall<V>(
        &mut self,
        subroutine_name: &'static str,
        arguments: V,
    ) -> &mut Self 
    where
        V: Iterator<Item = (char, A)>
    {
        // Yep, a pointer. Seriously, f*** borrowck.
        let subroutine: *const SubRoutine = self.get_subroutine(subroutine_name)
            .expect(format!(
                "There are no subroutines named \"{}\"",
                subroutine_name,
            ).as_str());

        let map = {
            let mut result = HashMap::with_capacity(arguments.size_hint().0);
            for (variable, argument) in arguments.take(unsafe {(*subroutine).variables.len()}) {
                assert!(
                    unsafe {(*subroutine).variables.contains(&variable)}, 
                    "No variable named '{}' in subroutine named \"{}\"",
                    variable,
                    subroutine_name,
                );
                result.insert(variable, argument);
            }
            result
        };

        for instruction in unsafe {(*subroutine).instructions.iter().rev()} {
            let target = *map.get(&instruction.target)
                .expect(format!(
                    "Can't match variable '{}' to a value: value not specified",
                    instruction.target,
                ).as_str());

            let control = instruction.control.map(|c| 
                *map.get(&c)
                    .expect(format!(
                        "Can't match variable '{}' to a value: value not specified",
                        c,
                    ).as_str())
            );

            self.push_instruction(
                instruction.gate_name,
                target,
                control,
                !instruction.reverse,
            );
        }

        self
    }
}

//#################################################################################################
//
//                                      SubRoutineBuilder
//
//#################################################################################################

pub struct SubRoutineBuilder<'a> {
    name: &'static str,
    variables: HashSet<char>,
    instructions: Vec<SingleInstruction<char>>,
    program: &'a mut ProgramBuilder<'a>,
    ended: bool,
}

impl<'a> SubRoutineBuilder<'a> {
    pub fn end(&mut self) -> &mut ProgramBuilder<'a> {
        assert!(
            !self.ended, 
            "SubRoutine has already been ended, cannot add more gates"
        );

        self.ended = true;

        let variables = {
            let mut result = HashSet::with_capacity(0);
            swap(&mut self.variables, &mut result);
            result
        };

        let instructions = {
            let mut result = Vec::with_capacity(0);
            swap(&mut self.instructions, &mut result);
            result.into()
        };

        self.program.subroutines.insert(self.name, SubRoutine {
            variables,
            instructions,
        });

        self.program
    }
}

impl private::InstructionChainInternals<char> for SubRoutineBuilder<'_> {
    fn push_instruction(
        &mut self,
        gate_name: &'static str, 
        target: char,
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
            gate_name,
            target,
            control,
            reverse,
        });
    }

    fn get_subroutine(
        &self,
        subroutine_name: &str,
    ) -> Option<&SubRoutine> {
        self.program.subroutines.get(subroutine_name)
    }
}

impl InstructionChain<char> for SubRoutineBuilder<'_> {}

//#################################################################################################
//
//                                      SubRoutine
//
//#################################################################################################

#[derive(Debug)]
pub struct SubRoutine {
    variables: HashSet<char>,
    instructions: Box<[SingleInstruction<char>]>,
}

//#################################################################################################
//
//                                       Program Builder
//
//#################################################################################################

/// A builder for the `Program` struct.
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
        assert!(
            samples != 0, 
            "Samples count cannot be 0"
        );

        let size = self.computer.size;

        self.measured = true;

        let initial_state = self.initial_state;

        let instructions = {
            let mut result = Vec::with_capacity(0);
            std::mem::swap(&mut self.instructions, &mut result);
            result.into()
        };

        let samples = samples;

        Program {
            size,
            initial_state,
            instructions,
            samples,
        }
    } 
}

impl private::InstructionChainInternals<Address> for ProgramBuilder<'_> {
    fn push_instruction(
        &mut self,
        gate_name: &'static str, 
        target: Address,
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
            gate_name,
            target,
            control,
            reverse,
        });
    }

    fn get_subroutine(
        &self,
        subroutine_name: &str,
    ) -> Option<&SubRoutine> {
        self.subroutines.get(subroutine_name)
    }
}

impl InstructionChain<Address> for ProgramBuilder<'_> {}

//#################################################################################################
//
//                                           Program
//
//#################################################################################################

#[derive(Debug)]
pub struct Program {
    size: Address,
    pub(crate) initial_state: usize,
    pub(crate) instructions: Box<[Instruction]>,
    pub(crate) samples: usize,
}

impl Program {
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "[\n  [Program with initial state |{:0size$b}>],\n  [Sample count of {}]",
            self.initial_state,
            size = self.size as usize,
        ).unwrap();

        let len = self.instructions.len();

        if len != 0 {
            write!(f, ",\n  [{} instructions:\n", len).unwrap();

            let dec = len.to_string().len() - 1;

            for (i, instruction) in self.instructions.iter().enumerate() {
                write!(f,
                    "    {:0dec$}: {} gate \"{}\" to qbit #{}{}{}\n",
                    i,
                    if instruction.reverse {"unapply"} else {"apply"},
                    instruction.gate_name,
                    instruction.target,
                    if let Some(control) = instruction.control {
                        format!(" with qbit #{} as control", control)
                    } else {
                        "".to_string()
                    },
                    if i+1 == len {""} else {","},
                    dec = dec,
                ).unwrap();
            }

            write!(f, "  ]\n]")
        } else {
            write!(f, "\n]")
        }
    }
}