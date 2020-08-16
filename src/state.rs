use crate::complex::c64;

use std::str::FromStr;

pub struct Register {
    qbit_count: usize,
    ket_size: usize,
    ket: Box<[c64]>,
}

//#################################################################################################
//
//                                       FromStr for State
//
//#################################################################################################

pub enum ParseRegisterError {
    MoreThan32Bits,
    WrongFormat,
    InvalidChar,
}

impl FromStr for Register {
    type Err = ();

    // Source string must match `|[01]{1-32}>`
    fn from_str(s: &str) -> Result<Register, ()> {
        if !s.is_ascii() {
            return Err(());
        }

        let qbit_count = s.len();
        if qbit_count < 3 && qbit_count > 34 {
            return Err(());
        }
        let mut register = Vec::with_capacity(qbit_count);

        let mut chars = s.chars();
        if chars.next().unwrap() != '|' {
            return Err(());
        }

        while let Some(c) = chars.next() {
            match c {
                '0' => register.push(false),
                '1' => register.push(true),
                '>' => break,
                _ => return Err(()),
            }
        }

        if chars.next() != None {
            return Err(());
        }

        let ket_size = 1 << (qbit_count - 1);

        Ok(Register {
            qbit_count,
            ket_size,
            ket: vec![].into(),
        })
    }
}
