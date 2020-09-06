use std::ops::{Index, Range, Shr};

use crate::computer::{QuantumComputer, Target};

pub struct Qbits<const N: usize>;

pub type Qbit = Qbits<1>;

pub struct DynQbits<'a> {
    computer: &'a mut QuantumComputer,
    qbits: Vec<usize>,
}

impl<'a> Shr<DynQbits<'a>> for DynQbits<'a> {
    type Output = &'a mut Target;

    fn shr(self, rhs: DynQbits) -> Self::Output {
        assert!(!self.qbits.iter().any(|x| rhs.qbits.contains(x)));
        self.computer.targets.extend(self.qbits);
        &mut self.computer.targets
    }
}

impl<'a> Shr<&[usize]> for &'a mut Target {
    type Output = &'a mut Target;

    fn shr(self, rhs: &[usize]) -> Self::Output {
        assert!(!self.0.iter().any(|x| rhs.contains(x)));
        self.0.extend(rhs);
        self
    }
}

impl Index<usize> for DynQbits<'_> {
    type Output = [usize];

    fn index(&self, index: usize) -> &[usize] {
        &[self.qbits[index]]
    }
}

impl Index<Range<usize>> for DynQbits<'_> {
    type Output = [usize];

    fn index(&self, index: Range<usize>) -> &[usize] {
        &self.qbits[index]
    }
}