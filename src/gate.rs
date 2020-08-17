use crate::register::Register;

pub(crate) union GateData {
    x: usize,
}

pub struct Gate {
    data: Box<GateData>,
    func: fn(&mut Register, &GateData),
}

impl Gate {
    pub(crate) fn new(data: Box<GateData>, func: fn(&mut Register, &GateData)) -> Gate {
        Gate { data, func }
    }

    pub fn apply(&self, register: &mut Register) {
        (self.func)(register, &self.data);
    }
}