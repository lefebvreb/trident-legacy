mod complex;
mod error;

pub mod computer;
pub mod gates;
pub mod program;

/*pub fn trivial() -> ocl::Result<()> {
    let src = include_str!("opencl/kernel.cl");

    let pro_que = ocl::ProQue::builder()
        .src(src)
        .dims(1 << 20)
        .build()?;

    let buffer = pro_que.create_buffer::<c64>()?;

    let kernel = pro_que.kernel_builder("add")
        .arg(&buffer)
        .arg(c64::new(5.0, -5.0))
        .build()?;

    unsafe { kernel.enq()?; }

    let mut vec = vec![c64::default(); buffer.len()];
    buffer.read(&mut vec).enq()?;

    println!("The value at index [{}] is now '{}'!", 513, vec[513]);
    Ok(())
}*/