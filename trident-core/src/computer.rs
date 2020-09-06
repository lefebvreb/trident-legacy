pub trait QuantumHardware {
    // TODO
}

 // MUST be cleared after each application, capacity is the size of the computer
pub struct Target(pub Vec<usize>);

pub struct QuantumComputer {
    hardware: Box<dyn QuantumHardware>,
    size: usize,
    pub targets: Target,
}

impl QuantumComputer {
    pub fn new(hardware: Box<dyn QuantumHardware>, size: usize) -> QuantumComputer {
        QuantumComputer {
            hardware,
            size,
            targets: Target(Vec::with_capacity(size)),
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }
}