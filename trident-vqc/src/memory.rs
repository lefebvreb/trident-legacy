use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
use std::pin::Pin;

//#################################################################################################
//
//                                         Memory
//
//#################################################################################################

// A struct holding all the references to the allocated buffer.
pub struct Memory<const NCHUNK: usize, const CHUNK_SIZE: usize> {
    chunks: Pin<Box<[Pin<Box<[u8]>>]>>,
}

impl<const NCHUNK: usize, const CHUNK_SIZE: usize> Memory<NCHUNK, CHUNK_SIZE> {
    // Interprets and prints the buffer
    pub fn debug<T: std::fmt::Debug>(&self) {
        print!("Memory {{ chunks: [");
        let chunks = unsafe {transmute::<_, &Box<[Box<[T]>]>>(&self.chunks)};
        let (imax, jmax) = (NCHUNK, CHUNK_SIZE / size_of::<T>());
        let commaif = |k, kmax| if k == kmax {""} else {", "};
        for i in 0..imax {
            print!("[");
            for j in 0..jmax {
                print!("{:?}{}", chunks[i][j], commaif(j, jmax));
            }
            print!("]{}", commaif(i, imax));
        }
        println!("] }}");
    }
}

//#################################################################################################
//
//                                      MemoryAccess
//
//#################################################################################################

// A struct providing read access to all of the buffer and write access
// to a small segment of the buffer.
#[derive(Copy, Clone)]
pub struct MemoryAccess<'a, const NCHUNK: usize, const CHUNK_SIZE: usize> {
    _p: PhantomData<&'a Memory<NCHUNK, CHUNK_SIZE>>,
    pointers: [*const u8; NCHUNK],
}

impl<const NCHUNK: usize, const CHUNK_SIZE: usize> MemoryAccess<'_, NCHUNK, CHUNK_SIZE> {
    const CHUNK_SIZE_LOG2: u32 = CHUNK_SIZE.trailing_zeros();
    const CHUNK_MASK: usize = CHUNK_SIZE - 1;

    pub const TOTAL_SIZE: usize = NCHUNK * CHUNK_SIZE;

    // Reads an element of type T from the buffer at the index
    // Overhead: 1 shl, 1 shr, 1 and, 1 add
    #[inline(always)]
    pub fn read<T: Copy>(self, mut index: usize) -> T {
        index *= size_of::<T>();
        let chunk = index.wrapping_shr(Self::CHUNK_SIZE_LOG2);
        unsafe {
            *(self.pointers[chunk].offset(
                (index & Self::CHUNK_MASK) as _
            ) as *const T)
        }
    }

    #[inline(always)]
    pub fn write<T>(self, mut index: usize, element: T) {
        index *= size_of::<T>();
        let chunk = index.wrapping_shr(Self::CHUNK_SIZE_LOG2);
        unsafe {
            *(self.pointers[chunk].offset(
                (index & Self::CHUNK_MASK) as _
            ) as *mut T) = element
        };
    }
}

//#################################################################################################
//
//                                         Constants
//
//#################################################################################################

pub struct Constants {
    pub nchunk: usize,
    pub chunk_size: usize,
    pub nwork: usize,
    pub work_size: usize,
}

impl Constants {
    pub fn calculate<T>(
        nelements: usize,
        max_mem_alloc: usize, 
        ncore: usize,
    ) -> Constants {
        debug_assert!(size_of::<T>().is_power_of_two());
        debug_assert!(nelements.is_power_of_two());
    
        let total_size = nelements * size_of::<T>();
        debug_assert!(total_size != 0);
    
        let (chunk_size, nchunk) = {
            let max = if max_mem_alloc.is_power_of_two() {
                max_mem_alloc
            } else {
                max_mem_alloc.wrapping_shr(1).next_power_of_two()
            };
            if total_size < max {
                (total_size, 1)
            } else {
                (max, total_size / max)
            }
        };
        debug_assert!(chunk_size * nchunk == total_size);
        debug_assert!(chunk_size.is_power_of_two());
        debug_assert!(chunk_size <= max_mem_alloc);
    
        let (work_size, nwork, work_per_chunk) = if ncore < nchunk {
            (chunk_size, nchunk, 1)
        } else {
            let div = ncore.next_power_of_two() / nchunk;
            (chunk_size / div, nchunk * div, div)
        };
        debug_assert!(work_size * work_per_chunk == chunk_size);
        debug_assert!(nwork / work_per_chunk == nchunk);
        debug_assert!(nwork >= ncore);
    
        Constants {
            nchunk,
            chunk_size,
            nwork,
            work_size,
        }
    }
}

//#################################################################################################
//
//                                           allocate
//
//#################################################################################################

// E: the type of elements to allocate, with a power of 2 for size
// nelements_log2: the number of elements to allocate (a power of 2)
// max_mem_alloc: the maximum size (in bytes) of a single chunk of memory
// ncore: the number of cores on the computer
pub fn allocate<
    'a,
    const NCHUNK: usize,
    const CHUNK_SIZE: usize,
>() -> (
    Memory<NCHUNK, CHUNK_SIZE>, 
    MemoryAccess<'a, NCHUNK, CHUNK_SIZE>
) {
    debug_assert!(CHUNK_SIZE.is_power_of_two());

    let chunks = (0..NCHUNK).map(
        |_| vec![0; CHUNK_SIZE].into_boxed_slice().into()
    ).collect::<Box<_>>().into();

    let memory = Memory {chunks};

    let chunks_refs = (*memory.chunks.iter().map(
        |chunk| chunk.as_ptr()
    ).collect::<Box<_>>()).try_into().unwrap();

    let access = MemoryAccess {
        _p: PhantomData,
        pointers: chunks_refs,
    };

    (memory, access)
}

//#################################################################################################
//
//                                             tests
//
//#################################################################################################

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation() {
        allocate::<'_, 1, 256>();
        allocate::<'_, 2, 32>();
    }
}
