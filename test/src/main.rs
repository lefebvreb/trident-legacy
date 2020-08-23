extern crate ocl;
use ocl::ProQue;

const SIZE: u8 = 4;

fn main() {
    let src = include_str!("kernels.cl");

    let pro_que = ProQue::builder()
        .src(src)
        .dims(1 << SIZE)
        .build().unwrap();

    let input = pro_que.create_buffer::<f32>().unwrap();
    let output = pro_que.buffer_builder::<f32>()
        .fill_val(Default::default())
        .flags(ocl::MemFlags::READ_WRITE)
        .build()
        .unwrap();

    let set = pro_que.kernel_builder("set")
        .arg(&input)
        .build().unwrap();

    let mut pyramid_sum = pro_que.kernel_builder("pyramid_sum")
        .arg(&input)
        .arg(&output)
        .arg(0usize)
        .arg(0usize)
        .build().unwrap();

    unsafe { set.enq().unwrap(); }

    let mut worksize: usize = 1 << SIZE;
    let mut offset = 0;
    for _ in 0..SIZE {
        worksize >>= 1;

        pyramid_sum
            .set_default_global_work_size(worksize.into())
            .set_default_global_work_offset(offset.into());

        pyramid_sum.set_arg(2, worksize as u64).unwrap();
        pyramid_sum.set_arg(3, offset as u64).unwrap();
        
        unsafe { pyramid_sum.enq().unwrap(); }

        offset += worksize;
    }

    let mut inp = vec![0.0; input.len()];
    input.read(&mut inp).enq().unwrap();
    println!("{:?}", inp);

    let mut out = vec![0.0; output.len()];
    output.read(&mut out).enq().unwrap();
    println!("{:?}", out);
}