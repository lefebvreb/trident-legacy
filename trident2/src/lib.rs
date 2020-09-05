#![allow(dead_code, unused_variables)]

extern crate num_complex;

mod memory;

pub fn test() {
    type T = f32;

    let (mem, mut accs) = memory::allocate::<T>(32, 32, 8);

    for acc in accs.drain(..) {
        let acc = Box::leak(acc);

        let offset = acc.work_offset::<T>();
        let size = acc.work_size::<T>();

        for index in offset..offset+size {
            let value = index as T / 2.0;
            acc.write(index, value);
        }
    }

    mem.debug::<T>();
}