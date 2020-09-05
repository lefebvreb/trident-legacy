use std::alloc::Layout;
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
use std::pin::Pin;

// A struct holding all the references to the allocated buffer.
pub(crate) struct Memory {
    chunks: Pin<Box<[Pin<Box<[u8]>>]>>,
}

// A struct providing read access to all of the buffer and write access
// to a small segment of the buffer.
#[repr(C)]
pub(crate) struct MemoryAccess<'a> {
    _p: PhantomData<&'a Memory>,
    work_offset: usize,
    work_size: usize,
    chunk_mask: usize,
    chunk_size_log2: usize,
    pointers: [*const u8],
}

// E: the type of elements to allocate, with a power of 2 for size
// nelements_log2: the number of elements to allocate (a power of 2)
// max_mem_alloc: the maximum size (in bytes) of a single chunk of memory
// ncore: the number of cores on the computer
pub(crate) fn allocate<'a, T>(
    nelements: usize,
    max_mem_alloc: usize, 
    ncore: usize,
) -> (Memory, Vec<Box<MemoryAccess<'a>>>) {
    assert!(size_of::<T>().is_power_of_two());
    assert!(nelements.is_power_of_two());

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

    let chunks = (0..nchunk).map(
        |_| vec![0; chunk_size].into_boxed_slice().into()
    ).collect::<Box<_>>().into();

    let memory = Memory {chunks};

    let chunks_refs = memory.chunks.iter().map(
        |chunk| chunk.as_ptr()
    ).collect::<Box<_>>();

    let size = Layout::from_size_align(0, 1).unwrap()
        .extend(Layout::new::<PhantomData<&'a Memory>>()).unwrap().0
        .extend(Layout::new::<usize>()).unwrap().0
        .extend(Layout::new::<usize>()).unwrap().0
        .extend(Layout::new::<usize>()).unwrap().0
        .extend(Layout::new::<usize>()).unwrap().0
        .extend(Layout::array::<*const u8>(nchunk).unwrap()).unwrap().0
        .pad_to_align().size();

    let bytes: Box<[u8]> = {
        let bytes: Box<[u8]> = vec![0; size].into();
        let mut access: Box<MemoryAccess<'a>> = unsafe {transmute(bytes)};

        access._p = PhantomData;
        access.work_size = work_size;
        access.chunk_mask = chunk_size - 1;
        access.chunk_size_log2 = chunk_size.trailing_zeros() as _;
        for i in 0..nchunk {
            access.pointers[i] = chunks_refs[i];
        }

        unsafe {transmute(access)}
    };

    let accesses = {
        let mut res = Vec::with_capacity(nwork);

        for chunk in 0..nchunk {
            let chunk_offset = chunk * chunk_size;
            for inwork in 0..work_per_chunk {
                let mut access: Box<MemoryAccess<'a>> = unsafe {transmute(bytes.clone())};
                access.work_offset = chunk_offset + inwork * work_size;
                res.push(access);
            }
        }

        res
    };

    (memory, accesses)
}

impl Memory {
    // Interprets and prints the buffer
    pub(crate) fn debug<T: std::fmt::Debug>(&self) {
        print!("Memory {{ chunks: [");
        let chunks = unsafe {transmute::<_, &Box<[Box<[T]>]>>(&self.chunks)};
        let (imax, jmax) = (self.chunks.len(), self.chunks[0].len() / size_of::<T>());
        for i in 0..imax {
            print!("[");
            for j in 0..jmax {
                print!("{:?}{}", chunks[i][j], if j == jmax-1 {""} else {", "});
            }
            print!("]{}", if i == imax-1 {""} else {", "});
        }
        println!("] }}");
    }
}

impl MemoryAccess<'_> {
    // Returns the work offset, the index of the first element covered by write accesses
    #[inline(always)]
    pub(crate) fn work_offset<T>(&self) -> usize {
        self.work_offset / size_of::<T>()
    }

    // Returns the work size, the number of elements covered by write accesses
    #[inline(always)]
    pub(crate) fn work_size<T>(&self) -> usize {
        self.work_size / size_of::<T>()
    }

    // Reads an element of type T from the buffer at the index
    // Overhead: 1 shl, 1 shr, 1 and, 1 add
    #[inline(always)]
    pub(crate) fn read<T: Copy>(&self, mut index: usize) -> T {
        index *= size_of::<T>();
        let j = index.wrapping_shr(self.chunk_size_log2 as _);
        unsafe {
            *(self.pointers[j].offset(
                (index & self.chunk_mask) as _
            ) as *const T)
        }
    }

    #[inline(always)]
    pub(crate) fn write<T>(&self, mut index: usize, element: T) {
        index *= size_of::<T>();
        let j = index.wrapping_shr(self.chunk_size_log2 as _);
        unsafe {
            *(self.pointers[j].offset(
                (index & self.chunk_mask) as _
            ) as *mut T) = element
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation() {
        allocate::<u128>(32, 1usize << 20, 8);
        allocate::<u64>(32, 2513, 7);
    }
}