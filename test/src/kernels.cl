__kernel void set(__global float *input) {
    input[get_global_id(0)] = 1.0;
}

__kernel void pyramid_sum(
    const __global float *input,
    __global float *output,
    const ulong worksize,
    const ulong offset
) {
    const size_t global_id = get_global_id(0);

    if (offset) {
        output[global_id] = output[global_id - worksize*2] + output[global_id - worksize*2+1];
    } else {
        output[global_id] = input[2*global_id] + input[2*global_id+1];
    }
}