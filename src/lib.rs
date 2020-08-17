#![feature(const_generics, test)]
#![allow(dead_code, incomplete_features)]

#[cfg(test)]
extern crate test;

extern crate packed_simd;

mod complex;

pub mod register;

pub mod gate;