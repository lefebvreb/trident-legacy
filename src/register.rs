use crate::complex::c128;

use std::fmt;

pub struct Register {
    qbit_count: usize,
    ket_size: usize,
    ket: Box<[c128]>,
}

impl Register {
    pub fn new(s: &str) -> Result<Register, &'static str> {
        if !s.is_ascii() {
            return Err("Source string must be ascii");
        }

        let n = s.len();
        if n < 3 {
            return Err("Source string must be at least 3 chars long");
        }
        if n > 62 {
            return Err("Source string must be at most 62 chars long");
        }

        let bytes = s.as_bytes();
        if bytes[0] as char != '|' {
            return Err("Source string must begin with a `|`");
        }
        if bytes[n-1] as char != '>' {
            return Err("Source string must end with a `>`");
        }

        let state = match usize::from_str_radix(s.get(1..n-1).unwrap(), 2) {
            Ok(state) => state,
            Err(_) => return Err("The state must be able to be represented by a usize"),
        };

        let qbit_count = n-2;
        let ket_size = 1 << qbit_count;
        let mut ket: Box<[c128]> = vec![c128::ZERO; ket_size].into();
        ket[state] = c128::ONE;

        Ok(Register {
            qbit_count,
            ket_size,
            ket,
        })
    }
}

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Register")
            .field("qbits", &self.qbit_count)
            .field("state", &self.ket)
            .finish()
    }
}